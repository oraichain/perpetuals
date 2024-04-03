use cosmwasm_std::{StdError, Uint128};
use cw20::Cw20ExecuteMsg;
use margined_perp::margined_engine::Side;
use margined_utils::{
    cw_multi_test::Executor,
    testing::{to_decimals, SimpleScenario},
};

use crate::testing::new_simple_scenario;

#[test]
fn test_increase_with_increase_position() {
    let SimpleScenario {
        mut router,
        alice,
        owner,
        engine,
        vamm,
        usdc,
        ..
    } = new_simple_scenario();

    let msg = vamm
        .set_open_interest_notional_cap(Uint128::from(600_000_000_000u128))
        .unwrap();
    router.execute(owner.clone(), msg).unwrap();

    // reduce the allowance
    router
        .execute_contract(
            alice.clone(),
            usdc.addr().clone(),
            &Cw20ExecuteMsg::DecreaseAllowance {
                spender: engine.addr().to_string(),
                amount: to_decimals(1400),
                expires: None,
            },
            &[],
        )
        .unwrap();

    let msg = engine
        .open_position(
            vamm.addr().to_string(),
            Side::Buy,
            to_decimals(600u64),
            to_decimals(1u64),
            Some(to_decimals(18)),
            Some(to_decimals(9)),
            to_decimals(0u64),
            vec![],
        )
        .unwrap();
    router.execute(alice.clone(), msg).unwrap();

    let open_interest_notional = engine.state(&router.wrap()).unwrap().open_interest_notional;
    assert_eq!(open_interest_notional, to_decimals(600u64));
}

#[test]
fn test_reduce_when_position_is_reduced() {
    let SimpleScenario {
        mut router,
        alice,
        owner,
        engine,
        vamm,
        usdc,
        ..
    } = new_simple_scenario();

    let msg = vamm
        .set_open_interest_notional_cap(Uint128::from(900_000_000_000u128))
        .unwrap();
    router.execute(owner.clone(), msg).unwrap();

    // reduce the allowance
    router
        .execute_contract(
            alice.clone(),
            usdc.addr().clone(),
            &Cw20ExecuteMsg::DecreaseAllowance {
                spender: engine.addr().to_string(),
                amount: to_decimals(1100),
                expires: None,
            },
            &[],
        )
        .unwrap();

    let msg = engine
        .open_position(
            vamm.addr().to_string(),
            Side::Buy,
            to_decimals(600u64),
            to_decimals(1u64),
            Some(to_decimals(18)),
            Some(to_decimals(9)),
            to_decimals(0u64),
            vec![],
        )
        .unwrap();
    router.execute(alice.clone(), msg).unwrap();

    let msg = engine
        .open_position(
            vamm.addr().to_string(),
            Side::Sell,
            to_decimals(300u64),
            to_decimals(1u64),
            Some(to_decimals(10)),
            Some(to_decimals(26)),
            to_decimals(0u64),
            vec![],
        )
        .unwrap();
    router.execute(alice.clone(), msg).unwrap();

    let open_interest_notional = engine.state(&router.wrap()).unwrap().open_interest_notional;
    assert_eq!(open_interest_notional, to_decimals(900u64));
}

#[test]
fn test_reduce_when_close_position() {
    let SimpleScenario {
        mut router,
        alice,
        owner,
        engine,
        vamm,
        usdc,
        ..
    } = new_simple_scenario();

    let msg = vamm
        .set_open_interest_notional_cap(Uint128::from(600_000_000_000u128))
        .unwrap();
    router.execute(owner.clone(), msg).unwrap();

    // reduce the allowance
    router
        .execute_contract(
            alice.clone(),
            usdc.addr().clone(),
            &Cw20ExecuteMsg::DecreaseAllowance {
                spender: engine.addr().to_string(),
                amount: to_decimals(1400),
                expires: None,
            },
            &[],
        )
        .unwrap();

    let msg = engine
        .open_position(
            vamm.addr().to_string(),
            Side::Buy,
            to_decimals(400u64),
            to_decimals(1u64),
            Some(to_decimals(18)),
            Some(to_decimals(9)),
            to_decimals(0u64),
            vec![],
        )
        .unwrap();
    router.execute(alice.clone(), msg).unwrap();

    let msg = engine
        .close_position(vamm.addr().to_string(), 1, to_decimals(0u64))
        .unwrap();
    router.execute(alice.clone(), msg).unwrap();

    let open_interest_notional = engine.state(&router.wrap()).unwrap().open_interest_notional;
    // this is near zero due to some rounding errors
    assert!(open_interest_notional < to_decimals(10u64));
}

#[test]
fn test_increase_when_traders_open_positions_in_diff_directions() {
    let SimpleScenario {
        mut router,
        alice,
        bob,
        owner,
        engine,
        vamm,
        usdc,
        ..
    } = new_simple_scenario();

    let msg = vamm
        .set_open_interest_notional_cap(Uint128::from(600_000_000_000u128))
        .unwrap();
    router.execute(owner.clone(), msg).unwrap();

    // reduce the allowance
    router
        .execute_contract(
            alice.clone(),
            usdc.addr().clone(),
            &Cw20ExecuteMsg::DecreaseAllowance {
                spender: engine.addr().to_string(),
                amount: to_decimals(1700),
                expires: None,
            },
            &[],
        )
        .unwrap();

    router
        .execute_contract(
            bob.clone(),
            usdc.addr().clone(),
            &Cw20ExecuteMsg::DecreaseAllowance {
                spender: engine.addr().to_string(),
                amount: to_decimals(1700),
                expires: None,
            },
            &[],
        )
        .unwrap();

    let msg = engine
        .open_position(
            vamm.addr().to_string(),
            Side::Buy,
            to_decimals(300u64),
            to_decimals(1u64),
            Some(to_decimals(18)),
            Some(to_decimals(9)),
            to_decimals(0u64),
            vec![],
        )
        .unwrap();
    router.execute(alice.clone(), msg).unwrap();

    let msg = engine
        .open_position(
            vamm.addr().to_string(),
            Side::Sell,
            to_decimals(300u64),
            to_decimals(1u64),
            Some(to_decimals(10)),
            Some(to_decimals(26)),
            to_decimals(0u64),
            vec![],
        )
        .unwrap();
    router.execute(bob.clone(), msg).unwrap();

    let open_interest_notional = engine.state(&router.wrap()).unwrap().open_interest_notional;
    assert_eq!(open_interest_notional, to_decimals(600u64));
}

#[test]
fn test_increase_when_traders_open_larger_positions_in_reverse_directions() {
    let SimpleScenario {
        mut router,
        alice,
        owner,
        engine,
        vamm,
        usdc,
        ..
    } = new_simple_scenario();

    let msg = vamm
        .set_open_interest_notional_cap(Uint128::from(600_000_000_000u128))
        .unwrap();
    router.execute(owner.clone(), msg).unwrap();

    // reduce the allowance
    router
        .execute_contract(
            alice.clone(),
            usdc.addr().clone(),
            &Cw20ExecuteMsg::DecreaseAllowance {
                spender: engine.addr().to_string(),
                amount: to_decimals(1400),
                expires: None,
            },
            &[],
        )
        .unwrap();

    let msg = engine
        .open_position(
            vamm.addr().to_string(),
            Side::Buy,
            to_decimals(250u64),
            to_decimals(1u64),
            Some(to_decimals(18)),
            Some(to_decimals(9)),
            to_decimals(0u64),
            vec![],
        )
        .unwrap();
    router.execute(alice.clone(), msg).unwrap();

    let msg = engine
        .open_position(
            vamm.addr().to_string(),
            Side::Sell,
            to_decimals(350u64),
            to_decimals(1u64),
            Some(to_decimals(10)),
            Some(to_decimals(17)),
            to_decimals(0u64),
            vec![],
        )
        .unwrap();
    router.execute(alice.clone(), msg).unwrap();

    let open_interest_notional = engine.state(&router.wrap()).unwrap().open_interest_notional;
    // this is near zero due to some rounding errors
    assert_eq!(open_interest_notional, to_decimals(600u64));
}

#[test]
fn test_zero_when_everyone_closes_positions() {
    let SimpleScenario {
        mut router,
        alice,
        bob,
        owner,
        engine,
        vamm,
        usdc,
        ..
    } = new_simple_scenario();

    let msg = vamm
        .set_open_interest_notional_cap(Uint128::from(600_000_000_000u128))
        .unwrap();
    router.execute(owner.clone(), msg).unwrap();

    // reduce the allowance
    router
        .execute_contract(
            alice.clone(),
            usdc.addr().clone(),
            &Cw20ExecuteMsg::DecreaseAllowance {
                spender: engine.addr().to_string(),
                amount: to_decimals(1400),
                expires: None,
            },
            &[],
        )
        .unwrap();

    router
        .execute_contract(
            bob.clone(),
            usdc.addr().clone(),
            &Cw20ExecuteMsg::DecreaseAllowance {
                spender: engine.addr().to_string(),
                amount: to_decimals(1400),
                expires: None,
            },
            &[],
        )
        .unwrap();

    let msg = engine
        .open_position(
            vamm.addr().to_string(),
            Side::Buy,
            to_decimals(250u64),
            to_decimals(1u64),
            Some(to_decimals(18)),
            Some(to_decimals(9)),
            to_decimals(0u64),
            vec![],
        )
        .unwrap();
    router.execute(alice.clone(), msg).unwrap();

    let msg = engine
        .open_position(
            vamm.addr().to_string(),
            Side::Sell,
            to_decimals(250u64),
            to_decimals(1u64),
            Some(to_decimals(12)),
            Some(to_decimals(17)),
            to_decimals(0u64),
            vec![],
        )
        .unwrap();
    router.execute(bob.clone(), msg).unwrap();

    let msg = engine
        .close_position(vamm.addr().to_string(), 1, to_decimals(0u64))
        .unwrap();
    router.execute(alice.clone(), msg).unwrap();

    let msg = engine
        .close_position(vamm.addr().to_string(), 2, to_decimals(0u64))
        .unwrap();
    router.execute(bob.clone(), msg).unwrap();

    let open_interest_notional = engine.state(&router.wrap()).unwrap().open_interest_notional;
    // this is near zero due to some rounding errors
    assert!(open_interest_notional < to_decimals(10u64));
}

#[test]
fn test_zero_when_everyone_closes_positions_one_position_is_bankrupt() {
    let SimpleScenario {
        mut router,
        alice,
        bob,
        owner,
        engine,
        vamm,
        usdc,
        ..
    } = new_simple_scenario();

    let msg = vamm
        .set_open_interest_notional_cap(Uint128::from(600_000_000_000u128))
        .unwrap();
    router.execute(owner.clone(), msg).unwrap();

    // reduce the allowance
    router
        .execute_contract(
            alice.clone(),
            usdc.addr().clone(),
            &Cw20ExecuteMsg::DecreaseAllowance {
                spender: engine.addr().to_string(),
                amount: to_decimals(1400),
                expires: None,
            },
            &[],
        )
        .unwrap();

    router
        .execute_contract(
            bob.clone(),
            usdc.addr().clone(),
            &Cw20ExecuteMsg::DecreaseAllowance {
                spender: engine.addr().to_string(),
                amount: to_decimals(1400),
                expires: None,
            },
            &[],
        )
        .unwrap();

    let msg = engine
        .open_position(
            vamm.addr().to_string(),
            Side::Sell,
            to_decimals(250u64),
            to_decimals(1u64),
            Some(to_decimals(6)),
            Some(to_decimals(13)),
            to_decimals(0u64),
            vec![],
        )
        .unwrap();
    router.execute(alice.clone(), msg).unwrap();

    let msg = engine
        .open_position(
            vamm.addr().to_string(),
            Side::Buy,
            to_decimals(250u64),
            to_decimals(1u64),
            Some(to_decimals(15)),
            Some(to_decimals(4)),
            to_decimals(0u64),
            vec![],
        )
        .unwrap();
    router.execute(bob.clone(), msg).unwrap();

    let msg = engine
        .liquidate(vamm.addr().to_string(), 1, to_decimals(0u64))
        .unwrap();
    router.execute(bob.clone(), msg).unwrap();

    router.update_block(|block| {
        block.time = block.time.plus_seconds(15);
        block.height += 1;
    });

    let msg = engine
        .close_position(vamm.addr().to_string(), 2, to_decimals(0u64))
        .unwrap();
    router.execute(bob.clone(), msg).unwrap();

    let open_interest_notional = engine.state(&router.wrap()).unwrap().open_interest_notional;
    // this is near zero due to some rounding errors
    assert!(open_interest_notional < to_decimals(10u64));
}

#[test]
fn test_open_interest_logged_without_cap() {
    let SimpleScenario {
        mut router,
        alice,
        bob,
        engine,
        vamm,
        ..
    } = new_simple_scenario();

    let msg = engine
        .open_position(
            vamm.addr().to_string(),
            Side::Sell,
            to_decimals(250u64),
            to_decimals(1u64),
            Some(to_decimals(6)),
            Some(to_decimals(13)),
            to_decimals(0u64),
            vec![],
        )
        .unwrap();
    router.execute(alice.clone(), msg).unwrap();

    let open_interest_notional = engine.state(&router.wrap()).unwrap().open_interest_notional;
    assert_eq!(open_interest_notional, Uint128::from(250_000_000_000u64));

    let msg = engine
        .open_position(
            vamm.addr().to_string(),
            Side::Buy,
            to_decimals(250u64),
            to_decimals(1u64),
            Some(to_decimals(10)),
            Some(to_decimals(4)),
            to_decimals(0u64),
            vec![],
        )
        .unwrap();
    router.execute(bob.clone(), msg).unwrap();

    let open_interest_notional = engine.state(&router.wrap()).unwrap().open_interest_notional;
    assert_eq!(open_interest_notional, Uint128::from(500_000_000_000u64));

    let msg = engine
        .liquidate(vamm.addr().to_string(), 1, to_decimals(0u64))
        .unwrap();
    router.execute(bob.clone(), msg).unwrap();

    router.update_block(|block| {
        block.time = block.time.plus_seconds(15);
        block.height += 1;
    });

    let msg = engine
        .close_position(vamm.addr().to_string(), 2, to_decimals(0u64))
        .unwrap();
    router.execute(bob.clone(), msg).unwrap();

    let open_interest_notional = engine.state(&router.wrap()).unwrap().open_interest_notional;
    // this is near zero due to some rounding errors
    assert!(open_interest_notional < to_decimals(10u64));
}

#[test]
fn test_stop_trading_if_over_open_interest_notional_cap() {
    let SimpleScenario {
        mut router,
        alice,
        bob,
        owner,
        engine,
        vamm,
        usdc,
        ..
    } = new_simple_scenario();

    let msg = vamm
        .set_open_interest_notional_cap(Uint128::from(600_000_000_000u128))
        .unwrap();
    router.execute(owner.clone(), msg).unwrap();

    // reduce the allowance
    router
        .execute_contract(
            alice.clone(),
            usdc.addr().clone(),
            &Cw20ExecuteMsg::DecreaseAllowance {
                spender: engine.addr().to_string(),
                amount: to_decimals(1400),
                expires: None,
            },
            &[],
        )
        .unwrap();

    router
        .execute_contract(
            bob.clone(),
            usdc.addr().clone(),
            &Cw20ExecuteMsg::DecreaseAllowance {
                spender: engine.addr().to_string(),
                amount: to_decimals(1400),
                expires: None,
            },
            &[],
        )
        .unwrap();

    let msg = engine
        .open_position(
            vamm.addr().to_string(),
            Side::Buy,
            to_decimals(600u64),
            to_decimals(1u64),
            Some(to_decimals(18)),
            Some(to_decimals(8)),
            to_decimals(0u64),
            vec![],
        )
        .unwrap();
    router.execute(alice.clone(), msg).unwrap();

    let msg = engine
        .open_position(
            vamm.addr().to_string(),
            Side::Buy,
            to_decimals(1u64),
            to_decimals(1u64),
            Some(to_decimals(27)),
            Some(to_decimals(25)),
            to_decimals(0u64),
            vec![],
        )
        .unwrap();
    let err = router.execute(bob.clone(), msg).unwrap_err();
    assert_eq!(
        StdError::GenericErr {
            msg: "open interest exceeds cap".to_string(),
        },
        err.downcast().unwrap()
    );
}

#[test]
fn test_wont_stop_trading_if_reducing_position_even_if_over_open_interest_notional_cap() {
    let SimpleScenario {
        mut router,
        alice,
        bob,
        owner,
        engine,
        vamm,
        usdc,
        ..
    } = new_simple_scenario();

    let msg = vamm
        .set_open_interest_notional_cap(Uint128::from(600_000_000_000u128))
        .unwrap();
    router.execute(owner.clone(), msg).unwrap();

    // reduce the allowance
    router
        .execute_contract(
            alice.clone(),
            usdc.addr().clone(),
            &Cw20ExecuteMsg::DecreaseAllowance {
                spender: engine.addr().to_string(),
                amount: to_decimals(1100),
                expires: None,
            },
            &[],
        )
        .unwrap();

    router
        .execute_contract(
            bob.clone(),
            usdc.addr().clone(),
            &Cw20ExecuteMsg::DecreaseAllowance {
                spender: engine.addr().to_string(),
                amount: to_decimals(1400),
                expires: None,
            },
            &[],
        )
        .unwrap();

    let msg = engine
        .open_position(
            vamm.addr().to_string(),
            Side::Buy,
            to_decimals(600u64),
            to_decimals(1u64),
            Some(to_decimals(18)),
            Some(to_decimals(8)),
            to_decimals(0u64),
            vec![],
        )
        .unwrap();
    router.execute(alice.clone(), msg).unwrap();

    let msg = vamm
        .set_open_interest_notional_cap(Uint128::from(900_000_000_000u128))
        .unwrap();
    router.execute(owner.clone(), msg).unwrap();

    let msg = engine
        .open_position(
            vamm.addr().to_string(),
            Side::Sell,
            to_decimals(300u64),
            to_decimals(1u64),
            Some(to_decimals(13)),
            Some(to_decimals(26)),
            to_decimals(0u64),
            vec![],
        )
        .unwrap();
    router.execute(alice.clone(), msg).unwrap();

    let open_interest_notional = engine.state(&router.wrap()).unwrap().open_interest_notional;
    assert_eq!(open_interest_notional, to_decimals(900u64));
}
