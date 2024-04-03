use cosmwasm_std::Uint128;
use cw20::Cw20ExecuteMsg;
use margined_perp::margined_engine::Side;
use margined_utils::{cw_multi_test::Executor, testing::to_decimals};

use crate::testing::new_simple_scenario;

#[test]
fn test_liquidator_can_open_position_and_liquidate_in_next_block() {
    let mut env = new_simple_scenario();

    // set the latest price
    let price = Uint128::from(10_000_000_000u128);
    let timestamp = env.router.block_info().time.seconds();

    let msg = env
        .pricefeed
        .append_price("ETH".to_string(), price, timestamp)
        .unwrap();
    env.router.execute(env.owner.clone(), msg).unwrap();

    env.router.update_block(|block| {
        block.time = block.time.plus_seconds(900);
        block.height += 1;
    });

    // set the margin ratios
    let msg = env
        .engine
        .set_margin_ratios(Uint128::from(100_000_000u128))
        .unwrap();
    env.router.execute(env.owner.clone(), msg).unwrap();

    let msg = env
        .engine
        .set_partial_liquidation_ratio(Uint128::from(250_000_000u128))
        .unwrap();
    env.router.execute(env.owner.clone(), msg).unwrap();

    let msg = env
        .engine
        .set_liquidation_fee(Uint128::from(25_000_000u128))
        .unwrap();
    env.router.execute(env.owner.clone(), msg).unwrap();

    // reduce the allowance
    env.router
        .execute_contract(
            env.alice.clone(),
            env.usdc.addr().clone(),
            &Cw20ExecuteMsg::DecreaseAllowance {
                spender: env.engine.addr().to_string(),
                amount: to_decimals(1000),
                expires: None,
            },
            &[],
        )
        .unwrap();

    // reduce the allowance
    env.router
        .execute_contract(
            env.bob.clone(),
            env.usdc.addr().clone(),
            &Cw20ExecuteMsg::DecreaseAllowance {
                spender: env.engine.addr().to_string(),
                amount: to_decimals(1000),
                expires: None,
            },
            &[],
        )
        .unwrap();

    // mint funds for carol
    env.router
        .execute_contract(
            env.owner.clone(),
            env.usdc.addr().clone(),
            &Cw20ExecuteMsg::Mint {
                recipient: env.carol.to_string(),
                amount: to_decimals(1000u64),
            },
            &[],
        )
        .unwrap();

    // set allowance for carol
    env.router
        .execute_contract(
            env.carol.clone(),
            env.usdc.addr().clone(),
            &Cw20ExecuteMsg::IncreaseAllowance {
                spender: env.engine.addr().to_string(),
                amount: to_decimals(1000u64),
                expires: None,
            },
            &[],
        )
        .unwrap();

    let msg = env
        .engine
        .open_position(
            env.vamm.addr().to_string(),
            Side::Buy,
            to_decimals(20u64),
            to_decimals(5u64),
            Some(to_decimals(18)),
            Some(Uint128::zero()),
            Uint128::from(9_090_000_000u128),
            vec![],
        )
        .unwrap();
    env.router.execute(env.bob.clone(), msg).unwrap();

    env.router.update_block(|block| {
        block.time = block.time.plus_seconds(15);
        block.height += 1;
    });

    let msg = env
        .engine
        .open_position(
            env.vamm.addr().to_string(),
            Side::Buy,
            to_decimals(20u64),
            to_decimals(5u64),
            Some(to_decimals(15)),
            Some(Uint128::zero()),
            Uint128::zero(),
            vec![],
            // Uint128::from(7_570_000_000u128),
        )
        .unwrap();
    env.router.execute(env.alice.clone(), msg).unwrap();

    env.router.update_block(|block| {
        block.time = block.time.plus_seconds(15);
        block.height += 1;
    });

    let msg = env
        .engine
        .open_position(
            env.vamm.addr().to_string(),
            Side::Sell,
            to_decimals(20u64),
            to_decimals(5u64),
            Some(to_decimals(5)),
            Some(to_decimals(16)),
            Uint128::zero(),
            vec![],
            // Uint128::from(7_580_000_000u128),
        )
        .unwrap();
    env.router.execute(env.bob.clone(), msg).unwrap();

    env.router.update_block(|block| {
        block.time = block.time.plus_seconds(15);
        block.height += 1;
    });

    let msg = env
        .engine
        .open_position(
            env.vamm.addr().to_string(),
            Side::Sell,
            to_decimals(20u64),
            to_decimals(5u64),
            Some(to_decimals(5)),
            Some(to_decimals(16)),
            to_decimals(0u64),
            vec![],
        )
        .unwrap();
    env.router.execute(env.carol.clone(), msg).unwrap();

    let price = env.vamm.spot_price(&env.router.wrap()).unwrap();
    let msg = env
        .pricefeed
        .append_price("ETH".to_string(), price, timestamp)
        .unwrap();
    env.router.execute(env.owner.clone(), msg).unwrap();

    let msg = env
        .engine
        .liquidate(env.vamm.addr().to_string(), 2, to_decimals(0u64))
        .unwrap();
    let response = env.router.execute(env.carol.clone(), msg).unwrap();
    assert_eq!(
        response.events[5].attributes[1].value,
        "partial_liquidation_reply".to_string()
    );
}

#[test]
fn test_can_open_position_short_and_liquidate_but_cannot_do_anything_more_in_same_block() {
    let mut env = new_simple_scenario();

    // set the latest price
    let price = Uint128::from(10_000_000_000u128);
    let timestamp = env.router.block_info().time.seconds();

    let msg = env
        .pricefeed
        .append_price("ETH".to_string(), price, timestamp)
        .unwrap();
    env.router.execute(env.owner.clone(), msg).unwrap();

    env.router.update_block(|block| {
        block.time = block.time.plus_seconds(900);
        block.height += 1;
    });

    // set the margin ratios
    let msg = env
        .engine
        .set_margin_ratios(Uint128::from(100_000_000u128))
        .unwrap();
    env.router.execute(env.owner.clone(), msg).unwrap();

    let msg = env
        .engine
        .set_partial_liquidation_ratio(Uint128::from(250_000_000u128))
        .unwrap();
    env.router.execute(env.owner.clone(), msg).unwrap();

    let msg = env
        .engine
        .set_liquidation_fee(Uint128::from(25_000_000u128))
        .unwrap();
    env.router.execute(env.owner.clone(), msg).unwrap();

    // reduce the allowance
    env.router
        .execute_contract(
            env.alice.clone(),
            env.usdc.addr().clone(),
            &Cw20ExecuteMsg::DecreaseAllowance {
                spender: env.engine.addr().to_string(),
                amount: to_decimals(1000),
                expires: None,
            },
            &[],
        )
        .unwrap();

    // reduce the allowance
    env.router
        .execute_contract(
            env.bob.clone(),
            env.usdc.addr().clone(),
            &Cw20ExecuteMsg::DecreaseAllowance {
                spender: env.engine.addr().to_string(),
                amount: to_decimals(1000),
                expires: None,
            },
            &[],
        )
        .unwrap();

    // mint funds for carol
    env.router
        .execute_contract(
            env.owner.clone(),
            env.usdc.addr().clone(),
            &Cw20ExecuteMsg::Mint {
                recipient: env.carol.to_string(),
                amount: to_decimals(1000u64),
            },
            &[],
        )
        .unwrap();

    // set allowance for carol
    env.router
        .execute_contract(
            env.carol.clone(),
            env.usdc.addr().clone(),
            &Cw20ExecuteMsg::IncreaseAllowance {
                spender: env.engine.addr().to_string(),
                amount: to_decimals(1000u64),
                expires: None,
            },
            &[],
        )
        .unwrap();

    let msg = env
        .engine
        .open_position(
            env.vamm.addr().to_string(),
            Side::Buy,
            to_decimals(20u64),
            to_decimals(5u64),
            Some(to_decimals(15)),
            Some(Uint128::zero()),
            Uint128::from(9_090_000_000u128),
            vec![],
        )
        .unwrap();
    env.router.execute(env.bob.clone(), msg).unwrap();

    env.router.update_block(|block| {
        block.time = block.time.plus_seconds(15);
        block.height += 1;
    });

    let msg = env
        .engine
        .open_position(
            env.vamm.addr().to_string(),
            Side::Buy,
            to_decimals(20u64),
            to_decimals(5u64),
            Some(to_decimals(15)),
            Some(Uint128::zero()),
            Uint128::from(7_570_000_000u128),
            vec![],
        )
        .unwrap();
    env.router.execute(env.alice.clone(), msg).unwrap();

    env.router.update_block(|block| {
        block.time = block.time.plus_seconds(15);
        block.height += 1;
    });

    let msg = env
        .engine
        .open_position(
            env.vamm.addr().to_string(),
            Side::Sell,
            to_decimals(20u64),
            to_decimals(5u64),
            Some(to_decimals(5)),
            Some(to_decimals(16)),
            Uint128::from(7_580_000_000u128),
            vec![],
        )
        .unwrap();
    env.router.execute(env.bob.clone(), msg).unwrap();

    env.router.update_block(|block| {
        block.time = block.time.plus_seconds(15);
        block.height += 1;
    });

    let msg = env
        .engine
        .open_position(
            env.vamm.addr().to_string(),
            Side::Sell,
            to_decimals(20u64),
            to_decimals(5u64),
            Some(to_decimals(5)),
            Some(to_decimals(16)),
            to_decimals(0u64),
            vec![],
        )
        .unwrap();
    env.router.execute(env.carol.clone(), msg).unwrap();

    let price = env.vamm.spot_price(&env.router.wrap()).unwrap();
    let msg = env
        .pricefeed
        .append_price("ETH".to_string(), price, timestamp)
        .unwrap();
    env.router.execute(env.owner.clone(), msg).unwrap();

    let msg = env
        .engine
        .liquidate(env.vamm.addr().to_string(), 2, to_decimals(0u64))
        .unwrap();
    env.router.execute(env.carol.clone(), msg).unwrap();

    let msg = env
        .engine
        .close_position(env.vamm.addr().to_string(), 4, to_decimals(0u64))
        .unwrap();
    let err = env.router.execute(env.carol.clone(), msg).unwrap_err();
    assert_eq!(
        err.source().unwrap().to_string(),
        "Generic error: Only one action allowed".to_string()
    );
}

#[test]
fn test_can_open_position_long_and_liquidate_but_cannot_do_anything_more_in_same_block() {
    let mut env = new_simple_scenario();

    // set the latest price
    let price = Uint128::from(10_000_000_000u128);
    let timestamp = env.router.block_info().time.seconds();

    let msg = env
        .pricefeed
        .append_price("ETH".to_string(), price, timestamp)
        .unwrap();
    env.router.execute(env.owner.clone(), msg).unwrap();

    env.router.update_block(|block| {
        block.time = block.time.plus_seconds(900);
        block.height += 1;
    });

    let msg = env
        .engine
        .set_margin_ratios(Uint128::from(100_000_000u128))
        .unwrap();
    env.router.execute(env.owner.clone(), msg).unwrap();

    let msg = env
        .engine
        .set_partial_liquidation_ratio(Uint128::from(250_000_000u128))
        .unwrap();
    env.router.execute(env.owner.clone(), msg).unwrap();

    let msg = env
        .engine
        .set_liquidation_fee(Uint128::from(25_000_000u128))
        .unwrap();
    env.router.execute(env.owner.clone(), msg).unwrap();

    // reduce the allowance
    env.router
        .execute_contract(
            env.alice.clone(),
            env.usdc.addr().clone(),
            &Cw20ExecuteMsg::DecreaseAllowance {
                spender: env.engine.addr().to_string(),
                amount: to_decimals(1000),
                expires: None,
            },
            &[],
        )
        .unwrap();

    // reduce the allowance
    env.router
        .execute_contract(
            env.bob.clone(),
            env.usdc.addr().clone(),
            &Cw20ExecuteMsg::DecreaseAllowance {
                spender: env.engine.addr().to_string(),
                amount: to_decimals(1000),
                expires: None,
            },
            &[],
        )
        .unwrap();

    // mint funds for carol
    env.router
        .execute_contract(
            env.owner.clone(),
            env.usdc.addr().clone(),
            &Cw20ExecuteMsg::Mint {
                recipient: env.carol.to_string(),
                amount: to_decimals(1000u64),
            },
            &[],
        )
        .unwrap();

    // set allowance for carol
    env.router
        .execute_contract(
            env.carol.clone(),
            env.usdc.addr().clone(),
            &Cw20ExecuteMsg::IncreaseAllowance {
                spender: env.engine.addr().to_string(),
                amount: to_decimals(1000u64),
                expires: None,
            },
            &[],
        )
        .unwrap();

    let msg = env
        .engine
        .open_position(
            env.vamm.addr().to_string(),
            Side::Sell,
            to_decimals(20u64),
            to_decimals(5u64),
            Some(to_decimals(5)),
            Some(to_decimals(16)),
            Uint128::zero(),
            vec![],
        )
        .unwrap();
    env.router.execute(env.bob.clone(), msg).unwrap();

    env.router.update_block(|block| {
        block.time = block.time.plus_seconds(15);
        block.height += 1;
    });

    let msg = env
        .engine
        .open_position(
            env.vamm.addr().to_string(),
            Side::Sell,
            to_decimals(20u64),
            to_decimals(5u64),
            Some(to_decimals(5)),
            Some(to_decimals(16)),
            Uint128::zero(),
            vec![],
        )
        .unwrap();
    env.router.execute(env.alice.clone(), msg).unwrap();

    env.router.update_block(|block| {
        block.time = block.time.plus_seconds(15);
        block.height += 1;
    });

    let msg = env
        .engine
        .close_position(env.vamm.addr().to_string(), 1, to_decimals(0u64))
        .unwrap();
    env.router.execute(env.bob.clone(), msg).unwrap();

    env.router.update_block(|block| {
        block.time = block.time.plus_seconds(15);
        block.height += 1;
    });

    let msg = env
        .engine
        .open_position(
            env.vamm.addr().to_string(),
            Side::Buy,
            to_decimals(20u64),
            to_decimals(5u64),
            Some(to_decimals(15)),
            Some(Uint128::zero()),
            to_decimals(0u64),
            vec![],
        )
        .unwrap();
    env.router.execute(env.carol.clone(), msg).unwrap();

    let price = env.vamm.spot_price(&env.router.wrap()).unwrap();
    let msg = env
        .pricefeed
        .append_price("ETH".to_string(), price, timestamp)
        .unwrap();
    env.router.execute(env.owner.clone(), msg).unwrap();

    let msg = env
        .engine
        .liquidate(env.vamm.addr().to_string(), 2, to_decimals(0u64))
        .unwrap();
    env.router.execute(env.carol.clone(), msg).unwrap();

    let msg = env
        .engine
        .close_position(env.vamm.addr().to_string(), 3, to_decimals(0u64))
        .unwrap();
    let err = env.router.execute(env.carol.clone(), msg).unwrap_err();
    assert_eq!(
        err.source().unwrap().to_string(),
        "Generic error: Only one action allowed".to_string()
    );
}

#[test]
fn test_can_open_position_and_liquidate_but_cannot_do_anything_more_in_same_block() {
    let mut env = new_simple_scenario();

    // set the latest price
    let price = Uint128::from(10_000_000_000u128);
    let timestamp = env.router.block_info().time.seconds();

    let msg = env
        .pricefeed
        .append_price("ETH".to_string(), price, timestamp)
        .unwrap();
    env.router.execute(env.owner.clone(), msg).unwrap();

    env.router.update_block(|block| {
        block.time = block.time.plus_seconds(900);
        block.height += 1;
    });

    let msg = env
        .engine
        .set_margin_ratios(Uint128::from(100_000_000u128))
        .unwrap();
    env.router.execute(env.owner.clone(), msg).unwrap();

    let msg = env
        .engine
        .set_partial_liquidation_ratio(Uint128::from(250_000_000u128))
        .unwrap();
    env.router.execute(env.owner.clone(), msg).unwrap();

    let msg = env
        .engine
        .set_liquidation_fee(Uint128::from(25_000_000u128))
        .unwrap();
    env.router.execute(env.owner.clone(), msg).unwrap();

    // reduce the allowance
    env.router
        .execute_contract(
            env.alice.clone(),
            env.usdc.addr().clone(),
            &Cw20ExecuteMsg::DecreaseAllowance {
                spender: env.engine.addr().to_string(),
                amount: to_decimals(1000),
                expires: None,
            },
            &[],
        )
        .unwrap();

    // reduce the allowance
    env.router
        .execute_contract(
            env.bob.clone(),
            env.usdc.addr().clone(),
            &Cw20ExecuteMsg::DecreaseAllowance {
                spender: env.engine.addr().to_string(),
                amount: to_decimals(1000),
                expires: None,
            },
            &[],
        )
        .unwrap();

    // mint funds for carol
    env.router
        .execute_contract(
            env.owner.clone(),
            env.usdc.addr().clone(),
            &Cw20ExecuteMsg::Mint {
                recipient: env.carol.to_string(),
                amount: to_decimals(1000u64),
            },
            &[],
        )
        .unwrap();

    // set allowance for carol
    env.router
        .execute_contract(
            env.carol.clone(),
            env.usdc.addr().clone(),
            &Cw20ExecuteMsg::IncreaseAllowance {
                spender: env.engine.addr().to_string(),
                amount: to_decimals(1000u64),
                expires: None,
            },
            &[],
        )
        .unwrap();

    let msg = env
        .engine
        .open_position(
            env.vamm.addr().to_string(),
            Side::Buy,
            to_decimals(20u64),
            to_decimals(5u64),
            Some(to_decimals(15)),
            Some(Uint128::zero()),
            Uint128::from(9_090_000_000u128),
            vec![],
        )
        .unwrap();
    env.router.execute(env.bob.clone(), msg).unwrap();

    env.router.update_block(|block| {
        block.time = block.time.plus_seconds(15);
        block.height += 1;
    });

    let msg = env
        .engine
        .open_position(
            env.vamm.addr().to_string(),
            Side::Buy,
            to_decimals(20u64),
            to_decimals(5u64),
            Some(to_decimals(15)),
            Some(Uint128::zero()),
            Uint128::from(7_570_000_000u128),
            vec![],
        )
        .unwrap();
    env.router.execute(env.alice.clone(), msg).unwrap();

    env.router.update_block(|block| {
        block.time = block.time.plus_seconds(15);
        block.height += 1;
    });

    let msg = env
        .engine
        .open_position(
            env.vamm.addr().to_string(),
            Side::Sell,
            to_decimals(20u64),
            to_decimals(5u64),
            Some(to_decimals(5)),
            Some(to_decimals(16)),
            Uint128::from(7_580_000_000u128),
            vec![],
        )
        .unwrap();
    env.router.execute(env.bob.clone(), msg).unwrap();

    env.router.update_block(|block| {
        block.time = block.time.plus_seconds(15);
        block.height += 1;
    });

    let msg = env
        .engine
        .open_position(
            env.vamm.addr().to_string(),
            Side::Buy,
            to_decimals(10u64),
            to_decimals(1u64),
            Some(to_decimals(15)),
            Some(Uint128::zero()),
            to_decimals(0u64),
            vec![],
        )
        .unwrap();
    env.router.execute(env.carol.clone(), msg).unwrap();

    let price = env.vamm.spot_price(&env.router.wrap()).unwrap();
    let msg = env
        .pricefeed
        .append_price("ETH".to_string(), price, timestamp)
        .unwrap();
    env.router.execute(env.owner.clone(), msg).unwrap();

    let msg = env
        .engine
        .liquidate(env.vamm.addr().to_string(), 2, to_decimals(0u64))
        .unwrap();
    env.router.execute(env.carol.clone(), msg).unwrap();

    let msg = env
        .engine
        .close_position(env.vamm.addr().to_string(), 4, to_decimals(0u64))
        .unwrap();
    let err = env.router.execute(env.carol.clone(), msg).unwrap_err();
    assert_eq!(
        err.source().unwrap().to_string(),
        "Generic error: Only one action allowed".to_string()
    );
}

#[test]
fn test_can_open_position_same_side_and_liquidate_but_cannot_do_anything_more_in_same_block() {
    let mut env = new_simple_scenario();

    // set the latest price
    let price = Uint128::from(10_000_000_000u128);
    let timestamp = env.router.block_info().time.seconds();

    let msg = env
        .pricefeed
        .append_price("ETH".to_string(), price, timestamp)
        .unwrap();
    env.router.execute(env.owner.clone(), msg).unwrap();

    env.router.update_block(|block| {
        block.time = block.time.plus_seconds(900);
        block.height += 1;
    });

    let msg = env
        .engine
        .set_margin_ratios(Uint128::from(100_000_000u128))
        .unwrap();
    env.router.execute(env.owner.clone(), msg).unwrap();

    let msg = env
        .engine
        .set_partial_liquidation_ratio(Uint128::from(250_000_000u128))
        .unwrap();
    env.router.execute(env.owner.clone(), msg).unwrap();

    let msg = env
        .engine
        .set_liquidation_fee(Uint128::from(25_000_000u128))
        .unwrap();
    env.router.execute(env.owner.clone(), msg).unwrap();

    // reduce the allowance
    env.router
        .execute_contract(
            env.alice.clone(),
            env.usdc.addr().clone(),
            &Cw20ExecuteMsg::DecreaseAllowance {
                spender: env.engine.addr().to_string(),
                amount: to_decimals(1000),
                expires: None,
            },
            &[],
        )
        .unwrap();

    // reduce the allowance
    env.router
        .execute_contract(
            env.bob.clone(),
            env.usdc.addr().clone(),
            &Cw20ExecuteMsg::DecreaseAllowance {
                spender: env.engine.addr().to_string(),
                amount: to_decimals(1000),
                expires: None,
            },
            &[],
        )
        .unwrap();

    // mint funds for carol
    env.router
        .execute_contract(
            env.owner.clone(),
            env.usdc.addr().clone(),
            &Cw20ExecuteMsg::Mint {
                recipient: env.carol.to_string(),
                amount: to_decimals(1000u64),
            },
            &[],
        )
        .unwrap();

    // set allowance for carol
    env.router
        .execute_contract(
            env.carol.clone(),
            env.usdc.addr().clone(),
            &Cw20ExecuteMsg::IncreaseAllowance {
                spender: env.engine.addr().to_string(),
                amount: to_decimals(1000u64),
                expires: None,
            },
            &[],
        )
        .unwrap();

    let msg = env
        .engine
        .open_position(
            env.vamm.addr().to_string(),
            Side::Sell,
            to_decimals(20u64),
            to_decimals(5u64),
            Some(to_decimals(5)),
            Some(to_decimals(16)),
            Uint128::zero(),
            vec![],
        )
        .unwrap();
    env.router.execute(env.bob.clone(), msg).unwrap();

    env.router.update_block(|block| {
        block.time = block.time.plus_seconds(15);
        block.height += 1;
    });

    let msg = env
        .engine
        .open_position(
            env.vamm.addr().to_string(),
            Side::Sell,
            to_decimals(20u64),
            to_decimals(5u64),
            Some(to_decimals(5)),
            Some(to_decimals(16)),
            Uint128::zero(),
            vec![],
        )
        .unwrap();
    env.router.execute(env.alice.clone(), msg).unwrap();

    env.router.update_block(|block| {
        block.time = block.time.plus_seconds(15);
        block.height += 1;
    });

    let msg = env
        .engine
        .close_position(env.vamm.addr().to_string(), 1, to_decimals(0u64))
        .unwrap();
    env.router.execute(env.bob.clone(), msg).unwrap();

    env.router.update_block(|block| {
        block.time = block.time.plus_seconds(15);
        block.height += 1;
    });

    let msg = env
        .engine
        .open_position(
            env.vamm.addr().to_string(),
            Side::Sell,
            to_decimals(10u64),
            to_decimals(1u64),
            Some(to_decimals(5)),
            Some(to_decimals(16)),
            to_decimals(0u64),
            vec![],
        )
        .unwrap();
    env.router.execute(env.carol.clone(), msg).unwrap();

    let price = env.vamm.spot_price(&env.router.wrap()).unwrap();
    let msg = env
        .pricefeed
        .append_price("ETH".to_string(), price, timestamp)
        .unwrap();
    env.router.execute(env.owner.clone(), msg).unwrap();

    let msg = env
        .engine
        .liquidate(env.vamm.addr().to_string(), 2, to_decimals(0u64))
        .unwrap();
    env.router.execute(env.carol.clone(), msg).unwrap();

    let msg = env
        .engine
        .close_position(env.vamm.addr().to_string(), 3, to_decimals(0u64))
        .unwrap();
    let err = env.router.execute(env.carol.clone(), msg).unwrap_err();
    assert_eq!(
        err.source().unwrap().to_string(),
        "Generic error: Only one action allowed".to_string()
    );
}
