use cosmwasm_std::Uint128;
use cw20::Cw20ExecuteMsg;
use margined_perp::margined_engine::Side;
use margined_utils::{
    cw_multi_test::Executor,
    testing::{to_decimals, SimpleScenario},
};

use crate::testing::new_simple_scenario;

#[test]
fn test_margin_engine_should_have_enough_balance_after_close_position() {
    let SimpleScenario {
        mut router,
        alice,
        bob,
        insurance_fund,
        engine,
        usdc,
        vamm,
        ..
    } = new_simple_scenario();

    // reduce the allowance
    router
        .execute_contract(
            alice.clone(),
            usdc.addr().clone(),
            &Cw20ExecuteMsg::DecreaseAllowance {
                spender: engine.addr().to_string(),
                amount: to_decimals(1900),
                expires: None,
            },
            &[],
        )
        .unwrap();

    // reduce the allowance
    router
        .execute_contract(
            bob.clone(),
            usdc.addr().clone(),
            &Cw20ExecuteMsg::DecreaseAllowance {
                spender: engine.addr().to_string(),
                amount: to_decimals(1800),
                expires: None,
            },
            &[],
        )
        .unwrap();

    // AMM after: 900 : 111.1111111111
    let msg = engine
        .open_position(
            vamm.addr().to_string(),
            Side::Sell,
            to_decimals(20u64),
            to_decimals(5u64),
            Some(to_decimals(6)),
            Some(to_decimals(15)),
            to_decimals(0u64),
            vec![],
        )
        .unwrap();
    router.execute(bob.clone(), msg).unwrap();

    // AMM after: 800 : 125
    let msg = engine
        .open_position(
            vamm.addr().to_string(),
            Side::Sell,
            to_decimals(25u64),
            to_decimals(4u64),
            Some(to_decimals(4)),
            Some(to_decimals(9)),
            to_decimals(0u64),
            vec![],
        )
        .unwrap();
    router.execute(alice.clone(), msg).unwrap();

    // 20(bob's margin) + 25(alice's margin) = 45
    let engine_balance = usdc.balance(&router.wrap(), engine.addr().clone()).unwrap();
    assert_eq!(engine_balance, to_decimals(45u64));

    // when bob close his position (11.11)
    // AMM after: 878.0487804877 : 113.8888888889
    // Bob's PnL = 21.951219512195121950
    // need to return Bob's margin 20 and PnL 21.951 = 41.951
    // clearingHouse balance: 45 - 41.951 = 3.048...
    let msg = engine
        .close_position(vamm.addr().to_string(), 1, to_decimals(0u64))
        .unwrap();
    router.execute(bob.clone(), msg).unwrap();

    let insurance_balance = usdc
        .balance(&router.wrap(), insurance_fund.addr().clone())
        .unwrap();
    assert_eq!(insurance_balance, to_decimals(5_000u64));

    let engine_balance = usdc.balance(&router.wrap(), engine.addr().clone()).unwrap();
    assert_eq!(engine_balance, Uint128::from(3_048_780_494u128));
}

#[test]
fn test_margin_engine_does_not_have_enough_balance_after_close_position() {
    let SimpleScenario {
        mut router,
        alice,
        bob,
        insurance_fund,
        engine,
        usdc,
        vamm,
        ..
    } = new_simple_scenario();

    // reduce the allowance
    router
        .execute_contract(
            alice.clone(),
            usdc.addr().clone(),
            &Cw20ExecuteMsg::DecreaseAllowance {
                spender: engine.addr().to_string(),
                amount: to_decimals(1900),
                expires: None,
            },
            &[],
        )
        .unwrap();

    // reduce the allowance
    router
        .execute_contract(
            bob.clone(),
            usdc.addr().clone(),
            &Cw20ExecuteMsg::DecreaseAllowance {
                spender: engine.addr().to_string(),
                amount: to_decimals(1800),
                expires: None,
            },
            &[],
        )
        .unwrap();

    // AMM after: 900 : 111.1111111111
    let msg = engine
        .open_position(
            vamm.addr().to_string(),
            Side::Sell,
            to_decimals(20u64),
            to_decimals(5u64),
            Some(to_decimals(7)),
            Some(to_decimals(15)),
            to_decimals(0u64),
            vec![],
        )
        .unwrap();
    router.execute(bob.clone(), msg).unwrap();

    // AMM after: 800 : 125
    let msg = engine
        .open_position(
            vamm.addr().to_string(),
            Side::Sell,
            to_decimals(20u64),
            to_decimals(5u64),
            Some(to_decimals(4)),
            Some(to_decimals(9)),
            to_decimals(0u64),
            vec![],
        )
        .unwrap();
    router.execute(alice.clone(), msg).unwrap();

    // 20(bob's margin) + 25(alice's margin) = 40
    let engine_balance = usdc.balance(&router.wrap(), engine.addr().clone()).unwrap();
    assert_eq!(engine_balance, to_decimals(40u64));

    // when bob close his position (11.11)
    // AMM after: 878.0487804877 : 113.8888888889
    // Bob's PnL = 21.951219512195121950
    // need to return Bob's margin 20 and PnL 21.951 = 41.951
    // clearingHouse balance: 40 - 41.951 = -1.95...
    let msg = engine
        .close_position(vamm.addr().to_string(), 1, to_decimals(0u64))
        .unwrap();
    router.execute(bob.clone(), msg).unwrap();

    let insurance_balance = usdc
        .balance(&router.wrap(), insurance_fund.addr().clone())
        .unwrap();
    assert_eq!(insurance_balance, Uint128::from(4998_048_780_494u128));

    let engine_balance = usdc.balance(&router.wrap(), engine.addr().clone()).unwrap();
    assert_eq!(engine_balance, Uint128::zero());
}
