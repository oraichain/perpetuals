use cosmwasm_std::{
    Addr, Deps, DepsMut, Env, Event, MessageInfo, Response, StdError, StdResult, Storage, SubMsg,
    SubMsgResponse, Uint128,
};
use margined_utils::contracts::helpers::{InsuranceFundController, VammController};

use std::str::FromStr;

use margined_common::{
    asset::{Asset, AssetInfo},
    integer::Integer,
};
use margined_perp::margined_engine::{
    PnlCalcOption, Position, PositionUnrealizedPnlResponse, RemainMarginResponse, Side,
};
use margined_perp::margined_vamm::Direction;

use crate::{
    contract::{PAUSER, WHITELIST},
    messages::execute_insurance_fund_withdrawal,
    query::query_cumulative_premium_fraction,
    state::{read_config, read_position, read_state, read_vamm_map, store_state, State},
};

// reads position from storage but also handles the case where there is no
// previously stored position, i.e. a new position
pub fn get_position(
    storage: &dyn Storage,
    position_key: &[u8],
    vamm: &Addr,
    trader: &Addr,
    side: &Side,
    block_height: u64,
) -> StdResult<Position> {
    // read the position for the trader from vamm
    let mut position = read_position(storage, &position_key)?;

    // so if the position returned is None then its new
    if position.vamm.as_str().is_empty() {
        // update the default position
        position.vamm = vamm.clone();
        position.trader = trader.clone();
        position.direction = side_to_direction(side);
        position.block_number = block_height;
    }

    Ok(position)
}

// Creates an asset from the eligible collateral and msg sent
pub fn get_asset(info: MessageInfo, eligible_collateral: AssetInfo) -> Asset {
    match &eligible_collateral {
        AssetInfo::Token { .. } => Asset {
            info: eligible_collateral,
            amount: Uint128::zero(),
        },
        AssetInfo::NativeToken { denom } => {
            let sent = match info.funds.iter().find(|&x| x.denom.eq(denom)) {
                Some(coin) => coin.amount,
                None => Uint128::zero(),
            };
            Asset {
                info: eligible_collateral,
                amount: sent,
            }
        }
    }
}

pub fn realize_bad_debt(
    deps: Deps,
    bad_debt: Uint128,
    messages: &mut Vec<SubMsg>,
    state: &mut State,
) -> StdResult<Uint128> {
    if state.prepaid_bad_debt > bad_debt {
        // no need to move extra tokens because vault already prepay bad debt, only need to update the numbers
        state.prepaid_bad_debt = state.prepaid_bad_debt.checked_sub(bad_debt)?;
    } else {
        // in order to realize all the bad debt vault need extra tokens from insuranceFund
        let bad_debt_delta = bad_debt.checked_sub(state.prepaid_bad_debt)?;

        messages.push(execute_insurance_fund_withdrawal(deps, bad_debt_delta)?);

        state.prepaid_bad_debt = Uint128::zero();

        return Ok(bad_debt_delta);
    };

    Ok(Uint128::zero())
}

// this blocks trades if open interest is too high, required during the bootstrapping of the project
pub fn update_open_interest_notional(
    deps: &Deps,
    state: &mut State,
    vamm: Addr,
    amount: Integer,
    trader: Addr,
) -> StdResult<Response> {
    let vamm_controller = VammController(vamm);
    let cap = vamm_controller
        .config(&deps.querier)?
        .open_interest_notional_cap;

    let mut updated_open_interest =
        amount.checked_add(Integer::new_positive(state.open_interest_notional))?;

    if updated_open_interest.is_negative() {
        updated_open_interest = Integer::zero();
    }

    // check if the cap has been exceeded - if trader address is in whitelist this bypasses
    if (!cap.is_zero()
        && amount.is_positive()
        && updated_open_interest > Integer::new_positive(cap))
        && !WHITELIST.query_hook(deps.to_owned(), trader.to_string())?
    {
        return Err(StdError::generic_err("open interest exceeds cap"));
    }

    state.open_interest_notional = updated_open_interest.value;

    Ok(Response::new())
}

// this blocks trades if user's position exceeds max base asset holding cap
pub fn check_base_asset_holding_cap(
    deps: &Deps,
    vamm: Addr,
    size: Uint128,
    trader: Addr,
) -> StdResult<Response> {
    let vamm_controller = VammController(vamm);
    let cap = vamm_controller
        .config(&deps.querier)?
        .base_asset_holding_cap;

    // check if the cap has been exceeded - if trader address is in whitelist this bypasses
    if (!cap.is_zero() && size > cap)
        && !WHITELIST.query_hook(deps.to_owned(), trader.to_string())?
    {
        return Err(StdError::generic_err("base asset holding exceeds cap"));
    }

    Ok(Response::new())
}

pub fn get_margin_ratio_calc_option(
    deps: Deps,
    vamm: String,
    trader: String,
    calc_option: PnlCalcOption,
) -> StdResult<Integer> {
    let config = read_config(deps.storage)?;
    let position_key = deps
        .api
        .keccak_256(&[vamm.as_bytes(), trader.as_bytes()].concat())?;
    // retrieve the latest position
    let position = read_position(deps.storage, &position_key)?;

    if position.size.is_zero() {
        return Ok(Integer::zero());
    }

    let PositionUnrealizedPnlResponse {
        position_notional,
        unrealized_pnl,
    } = get_position_notional_unrealized_pnl(deps, &position, calc_option)?;

    let remain_margin = calc_remain_margin_with_funding_payment(deps, position, unrealized_pnl)?;

    let margin_ratio = ((Integer::new_positive(remain_margin.margin)
        - Integer::new_positive(remain_margin.bad_debt))
        * Integer::new_positive(config.decimals))
        / Integer::new_positive(position_notional);

    Ok(margin_ratio)
}

pub fn get_position_notional_unrealized_pnl(
    deps: Deps,
    position: &Position,
    calc_option: PnlCalcOption,
) -> StdResult<PositionUnrealizedPnlResponse> {
    let mut output_notional = Uint128::zero();
    let mut unrealized_pnl = Integer::zero();

    let vamm_controller = VammController(position.vamm.clone());

    if !position.size.is_zero() {
        match calc_option {
            PnlCalcOption::Twap => {
                output_notional = vamm_controller.output_twap(
                    &deps.querier,
                    position.direction.clone(),
                    position.size.value,
                )?;
            }
            PnlCalcOption::SpotPrice => {
                output_notional = vamm_controller.output_amount(
                    &deps.querier,
                    position.direction.clone(),
                    position.size.value,
                )?;
            }
            PnlCalcOption::Oracle => {
                let config = read_config(deps.storage)?;
                let oracle_price = vamm_controller.underlying_price(&deps.querier)?;

                output_notional = oracle_price
                    .checked_mul(position.size.value)?
                    .checked_div(config.decimals)?;
            }
        }

        // we are short if the size of the position is less than 0
        unrealized_pnl = if position.direction == Direction::AddToAmm {
            Integer::new_positive(output_notional) - Integer::new_positive(position.notional)
        } else {
            Integer::new_positive(position.notional) - Integer::new_positive(output_notional)
        };
    }

    Ok(PositionUnrealizedPnlResponse {
        position_notional: output_notional,
        unrealized_pnl,
    })
}

pub fn calc_remain_margin_with_funding_payment(
    deps: Deps,
    position: Position,
    margin_delta: Integer,
) -> StdResult<RemainMarginResponse> {
    let config = read_config(deps.storage)?;

    // calculate the funding payment
    let latest_premium_fraction =
        query_cumulative_premium_fraction(deps, position.vamm.to_string())?;
    let funding_payment = (latest_premium_fraction - position.last_updated_premium_fraction)
        * position.size
        / Integer::new_positive(config.decimals);

    // calculate the remaining margin
    let mut remaining_margin: Integer =
        margin_delta - funding_payment + Integer::new_positive(position.margin);
    let mut bad_debt = Integer::zero();

    if remaining_margin.is_negative() {
        bad_debt = remaining_margin.invert_sign();
        remaining_margin = Integer::zero();
    }

    // if the remain is negative, set it to zero
    // and set the rest to
    Ok(RemainMarginResponse {
        funding_payment,
        margin: remaining_margin.value,
        bad_debt: bad_debt.value,
        latest_premium_fraction,
    })
}

// negative means trader pays and vice versa
pub fn calc_funding_payment(
    position: Position,
    latest_premium_fraction: Integer,
    decimals: Uint128,
) -> Integer {
    if !position.size.is_zero() {
        (latest_premium_fraction - position.last_updated_premium_fraction) * position.size
            / Integer::new_positive(decimals)
            * Integer::new_negative(1u64)
    } else {
        Integer::ZERO
    }
}

// this resets the main variables of a position
pub fn clear_position(env: Env, mut position: Position) -> StdResult<Position> {
    position.size = Integer::zero();
    position.margin = Uint128::zero();
    position.notional = Uint128::zero();
    position.last_updated_premium_fraction = Integer::zero();
    position.block_number = env.block.height;

    Ok(position)
}

pub fn update_pauser(deps: DepsMut, info: MessageInfo, pauser: String) -> StdResult<Response> {
    // validate the address
    let valid_pauser = deps.api.addr_validate(&pauser)?;

    PAUSER
        .execute_update_admin(deps, info, Some(valid_pauser))
        .map_err(|error| StdError::generic_err(format!("{}", error)))
}

// Adds an address to the whitelist for base asset holding cap
pub fn add_whitelist(deps: DepsMut, info: MessageInfo, address: String) -> StdResult<Response> {
    // validate the address
    let valid_addr = deps.api.addr_validate(&address)?;

    WHITELIST
        .execute_add_hook(&PAUSER, deps, info, valid_addr)
        .map_err(|error| StdError::generic_err(format!("{}", error)))
}

// Removes an address to the whitelist for base asset holding cap
pub fn remove_whitelist(deps: DepsMut, info: MessageInfo, address: String) -> StdResult<Response> {
    // validate the address
    let valid_addr = deps.api.addr_validate(&address)?;

    WHITELIST
        .execute_remove_hook(&PAUSER, deps, info, valid_addr)
        .map_err(|error| StdError::generic_err(format!("{}", error)))
}

pub fn set_pause(deps: DepsMut, _env: Env, info: MessageInfo, pause: bool) -> StdResult<Response> {
    let mut state = read_state(deps.storage)?;

    // check permission and if state matches
    // note: we could use `assert_admin` instead of `is_admin` except this would throw an `AdminError` and we would have to change the function sig
    if !PAUSER.is_admin(deps.as_ref(), &info.sender)? || state.pause == pause {
        return Err(StdError::generic_err("unauthorized"));
    }

    state.pause = pause;

    store_state(deps.storage, &state)?;

    Ok(Response::default().add_attribute("action", "set_pause"))
}

pub fn require_vamm(deps: Deps, insurance: &Option<Addr>, vamm: &Addr) -> StdResult<Response> {
    let insurance = match insurance {
        Some(arr) => arr,
        None => return Err(StdError::generic_err("insurance fund is not registered")),
    };

    let insurance_controller = InsuranceFundController(insurance.clone());

    // check that it is a registered vamm
    if !insurance_controller
        .is_vamm(&deps.querier, vamm.to_string())?
        .is_vamm
    {
        return Err(StdError::generic_err("vAMM is not registered"));
    }

    let vamm_controller = VammController(vamm.clone());
    // check that vamm is open
    if !vamm_controller.state(&deps.querier)?.open {
        return Err(StdError::generic_err("vAMM is not open"));
    }

    Ok(Response::new())
}

// Check no bad debt
pub fn require_bad_debt(bad_debt: Uint128) -> StdResult<Response> {
    if !bad_debt.is_zero() {
        return Err(StdError::generic_err("Insufficient margin"));
    }

    Ok(Response::new())
}

// Checks that position isn't zero
pub fn require_position_not_zero(size: Uint128) -> StdResult<Response> {
    if size.is_zero() {
        return Err(StdError::generic_err("Position is zero"));
    }

    Ok(Response::new())
}

// Checks that margin ratio is greater than base margin
pub fn require_additional_margin(
    margin_ratio: Integer,
    base_margin: Uint128,
) -> StdResult<Response> {
    if margin_ratio < Integer::new_positive(base_margin) {
        return Err(StdError::generic_err("Position is undercollateralized"));
    }

    Ok(Response::new())
}

pub fn require_insufficient_margin(
    margin_ratio: Integer,
    base_margin: Uint128,
) -> StdResult<Response> {
    if margin_ratio > Integer::new_positive(base_margin) {
        return Err(StdError::generic_err("Position is overcollateralized"));
    }

    Ok(Response::new())
}

pub fn require_not_restriction_mode(
    storage: &dyn Storage,
    position_key: &[u8],
    vamm: &Addr,
    block_height: u64,
) -> StdResult<Response> {
    let vamm_map = read_vamm_map(storage, vamm)?;
    let position = read_position(storage, &position_key)?;

    if vamm_map.last_restriction_block == block_height && position.block_number == block_height {
        return Err(StdError::generic_err("Only one action allowed"));
    }

    Ok(Response::new())
}

// check margin engine is not paused
pub fn require_not_paused(paused: bool) -> StdResult<Response> {
    if paused {
        return Err(StdError::generic_err("Margin engine is paused"));
    }

    Ok(Response::new())
}

// check an input is non-zero
pub fn require_non_zero_input(input: Uint128) -> StdResult<Response> {
    if input.is_zero() {
        return Err(StdError::generic_err("Input must be non-zero"));
    }

    Ok(Response::new())
}

pub fn parse_swap(response: SubMsgResponse) -> StdResult<(Uint128, Uint128)> {
    // Find swap inputs and output events
    let wasm = response
        .events
        .iter()
        .find(|&e| e.ty == "wasm")
        .ok_or_else(|| StdError::generic_err("No event found"))?;

    let swap = read_event("type".to_string(), wasm)?;

    match swap.as_str() {
        "input" => {
            let input_str = read_event("quote_asset_amount".to_string(), wasm)?;
            let output_str = read_event("base_asset_amount".to_string(), wasm)?;

            Ok((
                Uint128::from_str(&input_str)?,
                Uint128::from_str(&output_str)?,
            ))
        }
        "output" => {
            let input_str = read_event("base_asset_amount".to_string(), wasm)?;
            let output_str = read_event("quote_asset_amount".to_string(), wasm)?;

            Ok((
                Uint128::from_str(&input_str)?,
                Uint128::from_str(&output_str)?,
            ))
        }
        _ => {
            return Err(StdError::generic_err("Cannot parse swap"));
        }
    }
}

pub fn parse_pay_funding(response: SubMsgResponse) -> StdResult<(Integer, String)> {
    // Find swap inputs and output events
    let wasm = response
        .events
        .iter()
        .find(|&e| e.ty == "wasm")
        .ok_or_else(|| StdError::generic_err("No event found"))?;

    let premium_str = read_event("premium_fraction".to_string(), wasm)?;
    let premium: Integer = Integer::from_str(&premium_str)?;

    let sender = read_contract_address(wasm)?;

    Ok((premium, sender))
}

// Reads contract address from an event takes into account that there are
// inconistencies between cosmwasm versions and multitest etc...
fn read_contract_address(event: &Event) -> StdResult<String> {
    let result = event
        .attributes
        .iter()
        .find(|&attr| attr.key.eq("_contract_address"));

    result
        .ok_or_else(|| StdError::generic_err("No attribute found"))
        .map(|attr| attr.value.to_string())
}

fn read_event(key: String, event: &Event) -> StdResult<String> {
    let attr = event
        .attributes
        .iter()
        .find(|&attr| attr.key == key)
        .ok_or_else(|| StdError::generic_err("No attribute found"))?;

    Ok(attr.value.to_string())
}

// takes the side (buy|sell) and returns the direction (long|short)
pub fn side_to_direction(side: &Side) -> Direction {
    match side {
        Side::Buy => Direction::AddToAmm,
        Side::Sell => Direction::RemoveFromAmm,
    }
}

// takes the direction (long|short) and returns the side (buy|sell)
pub fn direction_to_side(direction: &Direction) -> Side {
    match direction {
        Direction::AddToAmm => Side::Buy,
        Direction::RemoveFromAmm => Side::Sell,
    }
}

pub fn position_to_side(size: Integer) -> Side {
    if size > Integer::zero() {
        Side::Sell
    } else {
        Side::Buy
    }
}
