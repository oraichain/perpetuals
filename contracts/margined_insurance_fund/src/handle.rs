use crate::{
    contract::OWNER,
    state::{read_config, read_vammlist, remove_vamm as remove_amm, save_vamm, VAMM_LIMIT},
};
use cosmwasm_std::{DepsMut, Env, MessageInfo, Response, StdError, StdResult, Uint128};

use margined_common::{asset::AssetInfo, messages::wasm_execute};
use margined_perp::margined_vamm::ExecuteMsg as VammExecuteMessage;
use margined_utils::contracts::helpers::{EngineController, VammController};

pub fn update_owner(deps: DepsMut, info: MessageInfo, owner: String) -> StdResult<Response> {
    // validate the address
    let valid_owner = deps.api.addr_validate(&owner)?;

    OWNER
        .execute_update_admin(deps, info, Some(valid_owner))
        .map_err(|error| StdError::generic_err(error.to_string()))
}

pub fn add_vamm(deps: DepsMut, info: MessageInfo, vamm: String) -> StdResult<Response> {
    let config = read_config(deps.storage)?;

    // check permission
    if !OWNER.is_admin(deps.as_ref(), &info.sender)? {
        return Err(StdError::generic_err("unauthorized"));
    }

    // validate address
    let vamm_valid = deps.api.addr_validate(&vamm)?;

    let vamm_controller = VammController(vamm_valid.clone());
    let engine_controller = EngineController(config.engine);

    // check decimals are consistent
    let engine_decimals = engine_controller.config(&deps.querier)?.decimals;
    let vamm_decimals = vamm_controller.config(&deps.querier)?.decimals;

    if engine_decimals != vamm_decimals {
        return Err(StdError::generic_err(
            "vAMM decimals incompatible with margin engine",
        ));
    }

    // add the amm
    save_vamm(deps.storage, vamm_valid)?;

    Ok(Response::default().add_attribute("action", "add_vamm"))
}

pub fn remove_vamm(deps: DepsMut, info: MessageInfo, vamm: String) -> StdResult<Response> {
    // check permission
    if !OWNER.is_admin(deps.as_ref(), &info.sender)? {
        return Err(StdError::generic_err("unauthorized"));
    }

    // validate address
    let vamm_valid = deps.api.addr_validate(&vamm)?;

    // remove vamm here
    remove_amm(deps.storage, vamm_valid)?;

    Ok(Response::default().add_attribute("action", "remove_amm"))
}

pub fn shutdown_all_vamm(deps: DepsMut, _env: Env, info: MessageInfo) -> StdResult<Response> {
    // check permission
    if !OWNER.is_admin(deps.as_ref(), &info.sender)? {
        return Err(StdError::generic_err("unauthorized"));
    }

    // construct all the shutdown messages
    let keys = read_vammlist(deps.storage, VAMM_LIMIT)?;

    // initialise the submsgs vec
    let mut msgs = vec![];
    for vamm in keys.iter() {
        let msg = wasm_execute(vamm, &VammExecuteMessage::SetOpen { open: false }, vec![])?;
        msgs.push(msg);
    }

    Ok(Response::default()
        .add_messages(msgs)
        .add_attribute("action", "shutdown_all_vamm"))
}

pub fn withdraw(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
    token: AssetInfo,
    amount: Uint128,
) -> StdResult<Response> {
    let config = read_config(deps.storage)?;

    // check permission
    if info.sender != config.engine {
        return Err(StdError::generic_err("unauthorized"));
    }

    // if eligible_collateral token balance can't afford the debt amount, ask staking token to mint
    let remain_amount = token.query_balance(&deps.querier, env.contract.address)?;
    // TODO: swap staking token to amount token using amm rounter
    if remain_amount < amount {
        // deposit_token is perp token, reward token default is perp token, and receive fee token distribution as well
        // insurance_fund contract is the minter of perp token
        // let required_perp_amount = query(router_addr, &SimulateSwapOperations { offer_amount: amount - remain_amount, operations: Vec<SwapOperation>})?
        // let required_perp_amount = swap_router_contract.simulate_swap(
        //     &deps.querier,
        //     amount - remain_amount,
        //     vec![SwapOperation::OraiSwap {
        //         offer_asset_info: staking_token,
        //         ask_asset_info: token,
        //     }])?;
        // mint_for_loss(required_perp_amount)
    }

    // send tokens if native or cw20
    let transfer_msg = token.into_msg(config.engine.to_string(), amount, None)?;

    Ok(Response::default()
        .add_message(transfer_msg)
        .add_attributes(vec![
            ("action", "insurance_withdraw"),
            ("amount", &amount.to_string()),
        ]))
}
