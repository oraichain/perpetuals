use cosmwasm_std::{StdError, Uint128};
use cw20::Cw20ExecuteMsg;
use margined_perp::margined_engine::{PnlCalcOption, Side};
use margined_utils::{
    cw_multi_test::Executor,
    testing::{to_decimals, SimpleScenario},
};

use crate::{contract::TRANSFER_FAILURE_REPLY_ID, testing::new_simple_scenario};

#[test]
fn test_open_position_total_fee_ten_percent() {
    let SimpleScenario {
        mut router,
        owner,
        alice,
        usdc,
        fee_pool,
        engine,
        vamm,
        insurance_fund,
        ..
    } = new_simple_scenario();

    // 10% fee
    let msg = vamm.set_toll_ratio(Uint128::from(50_000_000u128)).unwrap();
    router.execute(owner.clone(), msg).unwrap();

    let msg = vamm
        .set_spread_ratio(Uint128::from(50_000_000u128))
        .unwrap();
    router.execute(owner.clone(), msg).unwrap();

    // given 300 x 2 quote asset, get 37.5 base asset
    // fee is 300 x 2 x 10% = 60
    // user needs to pay 300 + 60 = 360
    let msg = engine
        .open_position(
            vamm.addr().to_string(),
            Side::Buy,
            Uint128::from(300_000_000_000u64),
            Uint128::from(2_000_000_000u64),
            Some(to_decimals(18)),
            Some(Uint128::zero()),
            Uint128::from(17_500_000_000u64),
            vec![],
        )
        .unwrap();
    router.execute(alice.clone(), msg).unwrap();

    let alice_position = engine
        .get_position_with_funding_payment(&router.wrap(), vamm.addr().to_string(), 1)
        .unwrap();
    assert_eq!(alice_position.margin, Uint128::from(240_000_000_000u64));

    let engine_balance = usdc.balance(&router.wrap(), engine.addr().clone()).unwrap();
    assert_eq!(engine_balance, Uint128::from(240_000_000_000u128));

    let fee_pool_balance = usdc
        .balance(&router.wrap(), fee_pool.addr().clone())
        .unwrap();
    assert_eq!(fee_pool_balance, Uint128::from(30_000_000_000u128));

    let insurance_balance = usdc
        .balance(&router.wrap(), insurance_fund.addr().clone())
        .unwrap();
    assert_eq!(insurance_balance, Uint128::from(5030_000_000_000u128));
}

#[test]
fn test_open_short_position_twice_total_fee_ten_percent() {
    let SimpleScenario {
        mut router,
        owner,
        alice,
        usdc,
        fee_pool,
        engine,
        vamm,
        insurance_fund,
        ..
    } = new_simple_scenario();

    // 10% fee
    let msg = vamm.set_toll_ratio(Uint128::from(50_000_000u128)).unwrap();
    router.execute(owner.clone(), msg).unwrap();

    let msg = vamm
        .set_spread_ratio(Uint128::from(50_000_000u128))
        .unwrap();
    router.execute(owner.clone(), msg).unwrap();

    // given 50 x 2 quote asset, get 11.1 base asset
    // fee is 50 x 2 x 10% = 10
    // user needs to pay 50 + 10 = 60
    let msg = engine
        .open_position(
            vamm.addr().to_string(),
            Side::Sell,
            Uint128::from(50_000_000_000u64),
            Uint128::from(2_000_000_000u64),
            Some(to_decimals(4)),
            Some(to_decimals(10)),
            Uint128::from(11_200_000_000u64),
            vec![],
        )
        .unwrap();
    router.execute(alice.clone(), msg).unwrap();

    let alice_balance_1 = usdc.balance(&router.wrap(), alice.clone()).unwrap();

    let msg = engine
        .open_position(
            vamm.addr().to_string(),
            Side::Sell,
            Uint128::from(50_000_000_000u64),
            Uint128::from(2_000_000_000u64),
            Some(to_decimals(4)),
            Some(to_decimals(10)),
            Uint128::from(139_000_000_000u64),
            vec![],
        )
        .unwrap();
    router.execute(alice.clone(), msg).unwrap();

    let alice_balance_2 = usdc.balance(&router.wrap(), alice.clone()).unwrap();
    assert_eq!(
        alice_balance_1 - alice_balance_2,
        Uint128::from(50_000_000_000u64)
    );

    let engine_balance = usdc.balance(&router.wrap(), engine.addr().clone()).unwrap();
    assert_eq!(engine_balance, Uint128::from(80_000_000_000u128));

    let fee_pool_balance = usdc
        .balance(&router.wrap(), fee_pool.addr().clone())
        .unwrap();
    assert_eq!(fee_pool_balance, Uint128::from(10_000_000_000u128));

    let insurance_balance = usdc
        .balance(&router.wrap(), insurance_fund.addr().clone())
        .unwrap();
    assert_eq!(insurance_balance, Uint128::from(5010_000_000_000u128));
}

#[test]
fn test_open_and_close_position_fee_ten_percent() {
    let SimpleScenario {
        mut router,
        owner,
        alice,
        usdc,
        fee_pool,
        engine,
        vamm,
        insurance_fund,
        ..
    } = new_simple_scenario();

    // 10% fee
    let msg = vamm.set_toll_ratio(Uint128::from(50_000_000u128)).unwrap();
    router.execute(owner.clone(), msg).unwrap();

    let msg = vamm
        .set_spread_ratio(Uint128::from(50_000_000u128))
        .unwrap();
    router.execute(owner.clone(), msg).unwrap();

    // given 50 x 2 quote asset, get 11.1 base asset
    // fee is 50 x 2 x 10% = 10
    // user needs to pay 50 + 10 = 60
    let msg = engine
        .open_position(
            vamm.addr().to_string(),
            Side::Buy,
            Uint128::from(300_000_000_000u64),
            Uint128::from(2_000_000_000u64),
            Some(to_decimals(18)),
            Some(Uint128::zero()),
            Uint128::from(17_500_000_000u64),
            vec![],
        )
        .unwrap();
    router.execute(alice.clone(), msg).unwrap();

    let msg = engine
        .close_position(vamm.addr().to_string(), 1, Uint128::zero())
        .unwrap();
    router.execute(alice.clone(), msg).unwrap();

    let engine_balance = usdc.balance(&router.wrap(), engine.addr().clone()).unwrap();
    assert_eq!(engine_balance, Uint128::from(7u128));

    let fee_pool_balance = usdc
        .balance(&router.wrap(), fee_pool.addr().clone())
        .unwrap();
    assert_eq!(fee_pool_balance, Uint128::from(60_000_000_000u128));

    let insurance_balance = usdc
        .balance(&router.wrap(), insurance_fund.addr().clone())
        .unwrap();
    assert_eq!(insurance_balance, Uint128::from(5060_000_000_000u128));
}

#[test]
fn test_open_position_close_manually_open_reverse_position_total_fee_ten_percent() {
    let SimpleScenario {
        mut router,
        owner,
        alice,
        usdc,
        fee_pool,
        engine,
        vamm,
        insurance_fund,
        ..
    } = new_simple_scenario();

    // 10% fee
    let msg = vamm.set_toll_ratio(Uint128::from(50_000_000u128)).unwrap();
    router.execute(owner.clone(), msg).unwrap();

    let msg = vamm
        .set_spread_ratio(Uint128::from(50_000_000u128))
        .unwrap();
    router.execute(owner.clone(), msg).unwrap();

    let msg = engine
        .open_position(
            vamm.addr().to_string(),
            Side::Buy,
            Uint128::from(300_000_000_000u64),
            Uint128::from(2_000_000_000u64),
            Some(to_decimals(18)),
            Some(Uint128::zero()),
            Uint128::zero(),
            vec![],
        )
        .unwrap();
    router.execute(alice.clone(), msg).unwrap();

    let pnl = engine
        .get_unrealized_pnl(
            &router.wrap(),
            vamm.addr().to_string(),
            1,
            PnlCalcOption::SpotPrice,
        )
        .unwrap();

    let msg = engine
        .open_position(
            vamm.addr().to_string(),
            Side::Sell,
            pnl.position_notional,
            Uint128::from(1_000_000_000u64),
            Some(to_decimals(4)),
            Some(to_decimals(30)),
            Uint128::zero(),
            vec![],
        )
        .unwrap();
    router.execute(alice.clone(), msg).unwrap();

    // 1st tx fee = 300 * 2 * 5% = 30
    // 1st tx spread = 300 * 2 * 5% = 30
    // 2nd tx fee = 300 * 2 * 5% = 30
    // 2nd tx fee = 300 * 2 * 5% = 30
    let fee_pool_balance = usdc
        .balance(&router.wrap(), fee_pool.addr().clone())
        .unwrap();
    assert_eq!(fee_pool_balance, Uint128::from(53_999_999_999u128));

    let insurance_balance = usdc
        .balance(&router.wrap(), insurance_fund.addr().clone())
        .unwrap();
    assert_eq!(insurance_balance, Uint128::from(5_053_999_999_999u128));
}

#[test]
fn test_open_position_close_manually_open_reverse_position_short_then_long_total_fee_ten_percent() {
    let SimpleScenario {
        mut router,
        owner,
        alice,
        usdc,
        fee_pool,
        engine,
        vamm,
        insurance_fund,
        ..
    } = new_simple_scenario();

    // 10% fee
    let msg = vamm.set_toll_ratio(Uint128::from(50_000_000u128)).unwrap();
    router.execute(owner.clone(), msg).unwrap();

    let msg = vamm
        .set_spread_ratio(Uint128::from(50_000_000u128))
        .unwrap();
    router.execute(owner.clone(), msg).unwrap();

    let msg = engine
        .open_position(
            vamm.addr().to_string(),
            Side::Sell,
            Uint128::from(300_000_000_000u64),
            Uint128::from(2_000_000_000u64),
            Some(to_decimals(3)),
            Some(to_decimals(10)),
            Uint128::zero(),
            vec![],
        )
        .unwrap();
    router.execute(alice.clone(), msg).unwrap();

    let pnl = engine
        .get_unrealized_pnl(
            &router.wrap(),
            vamm.addr().to_string(),
            1,
            PnlCalcOption::SpotPrice,
        )
        .unwrap();

    let msg = engine
        .open_position(
            vamm.addr().to_string(),
            Side::Buy,
            pnl.position_notional,
            Uint128::from(1_000_000_000u64),
            Some(to_decimals(16)),
            Some(to_decimals(1)),
            Uint128::zero(),
            vec![],
        )
        .unwrap();
    router.execute(alice.clone(), msg).unwrap();

    // 1st tx fee = 300 * 2 * 5% = 30
    // 1st tx spread = 300 * 2 * 5% = 30
    // 2nd tx fee = 300 * 2 * 5% = 30
    // 2nd tx fee = 300 * 2 * 5% = 30
    let fee_pool_balance = usdc
        .balance(&router.wrap(), fee_pool.addr().clone())
        .unwrap();
    assert_eq!(fee_pool_balance, Uint128::from(54_000_000_000u128));

    let insurance_balance = usdc
        .balance(&router.wrap(), insurance_fund.addr().clone())
        .unwrap();
    assert_eq!(insurance_balance, Uint128::from(5054_000_000_000u128));
}

#[test]
fn test_open_position_reduce_position_total_fee_ten_percent() {
    let SimpleScenario {
        mut router,
        owner,
        alice,
        usdc,
        fee_pool,
        engine,
        vamm,
        insurance_fund,
        ..
    } = new_simple_scenario();

    // 10% fee
    let msg = vamm.set_toll_ratio(Uint128::from(50_000_000u128)).unwrap();
    router.execute(owner.clone(), msg).unwrap();

    let msg = vamm
        .set_spread_ratio(Uint128::from(50_000_000u128))
        .unwrap();
    router.execute(owner.clone(), msg).unwrap();

    let msg = engine
        .open_position(
            vamm.addr().to_string(),
            Side::Buy,
            Uint128::from(300_000_000_000u64),
            Uint128::from(2_000_000_000u64),
            Some(to_decimals(18)),
            Some(Uint128::zero()),
            Uint128::zero(),
            vec![],
        )
        .unwrap();
    router.execute(alice.clone(), msg).unwrap();

    let msg = engine
        .open_position(
            vamm.addr().to_string(),
            Side::Sell,
            Uint128::from(300_000_000_000u64),
            Uint128::from(1_000_000_000u64),
            Some(to_decimals(7)),
            Some(to_decimals(30)),
            Uint128::zero(),
            vec![],
        )
        .unwrap();
    router.execute(alice.clone(), msg).unwrap();

    // 1st tx fee = 300 * 2 * 5% = 30
    // 1st tx spread = 300 * 2 * 5% = 30
    // 2nd tx fee = 300 * 2 * 5% = 15
    // 2nd tx fee = 300 * 2 * 5% = 15
    let fee_pool_balance = usdc
        .balance(&router.wrap(), fee_pool.addr().clone())
        .unwrap();
    assert_eq!(fee_pool_balance, Uint128::from(45_000_000_000u128));

    let insurance_balance = usdc
        .balance(&router.wrap(), insurance_fund.addr().clone())
        .unwrap();
    assert_eq!(insurance_balance, Uint128::from(5045_000_000_000u128));
}

#[test]
fn test_open_position_reduce_position_short_then_long_total_fee_ten_percent() {
    let SimpleScenario {
        mut router,
        owner,
        alice,
        usdc,
        fee_pool,
        engine,
        vamm,
        insurance_fund,
        ..
    } = new_simple_scenario();

    // 10% fee
    let msg = vamm.set_toll_ratio(Uint128::from(50_000_000u128)).unwrap();
    router.execute(owner.clone(), msg).unwrap();

    let msg = vamm
        .set_spread_ratio(Uint128::from(50_000_000u128))
        .unwrap();
    router.execute(owner.clone(), msg).unwrap();

    let msg = engine
        .open_position(
            vamm.addr().to_string(),
            Side::Sell,
            Uint128::from(300_000_000_000u64),
            Uint128::from(2_000_000_000u64),
            Some(to_decimals(3)),
            Some(to_decimals(7)),
            Uint128::zero(),
            vec![],
        )
        .unwrap();
    router.execute(alice.clone(), msg).unwrap();

    let msg = engine
        .open_position(
            vamm.addr().to_string(),
            Side::Buy,
            Uint128::from(300_000_000_000u128),
            Uint128::from(1_000_000_000u64),
            Some(to_decimals(15)),
            Some(Uint128::zero()),
            Uint128::zero(),
            vec![],
        )
        .unwrap();
    router.execute(alice.clone(), msg).unwrap();

    // 1st tx fee = 300 * 2 * 5% = 30
    // 1st tx spread = 300 * 2 * 5% = 30
    // 2nd tx fee = 150 * 2 * 5% = 15
    // 2nd tx fee = 150 * 2 * 5% = 15
    let fee_pool_balance = usdc
        .balance(&router.wrap(), fee_pool.addr().clone())
        .unwrap();
    assert_eq!(fee_pool_balance, Uint128::from(45_000_000_000u128));

    let insurance_balance = usdc
        .balance(&router.wrap(), insurance_fund.addr().clone())
        .unwrap();
    assert_eq!(insurance_balance, Uint128::from(5045_000_000_000u128));
}

#[test]
fn test_close_under_collateral_position_total_fee_ten_percent() {
    let SimpleScenario {
        mut router,
        owner,
        alice,
        bob,
        usdc,
        fee_pool,
        engine,
        vamm,
        ..
    } = new_simple_scenario();

    // 10% fee
    let msg = vamm.set_toll_ratio(Uint128::from(50_000_000u128)).unwrap();
    router.execute(owner.clone(), msg).unwrap();

    let msg = vamm
        .set_spread_ratio(Uint128::from(50_000_000u128))
        .unwrap();
    router.execute(owner.clone(), msg).unwrap();

    // reduce the allowance to 60
    router
        .execute_contract(
            alice.clone(),
            usdc.addr().clone(),
            &Cw20ExecuteMsg::DecreaseAllowance {
                spender: engine.addr().to_string(),
                amount: Uint128::from(1_940_000_000_000u64),
                expires: None,
            },
            &[],
        )
        .unwrap();

    let msg = engine
        .open_position(
            vamm.addr().to_string(),
            Side::Buy,
            Uint128::from(20_000_000_000u64),
            Uint128::from(5_000_000_000u64),
            Some(to_decimals(15)),
            Some(to_decimals(8)),
            Uint128::zero(),
            vec![],
        )
        .unwrap();
    router.execute(alice.clone(), msg).unwrap();

    let msg = engine
        .open_position(
            vamm.addr().to_string(),
            Side::Sell,
            Uint128::from(40_000_000_000u64),
            Uint128::from(5_000_000_000u64),
            Some(to_decimals(3)),
            Some(to_decimals(30)),
            Uint128::zero(),
            vec![],
        )
        .unwrap();
    router.execute(bob.clone(), msg).unwrap();

    let msg = engine
        .liquidate(vamm.addr().to_string(), 1, Uint128::zero())
        .unwrap();
    let tx = router.execute(bob.clone(), msg).unwrap();
    println!("liquidate tx: {:?}", tx);

    let fee_pool_balance = usdc
        .balance(&router.wrap(), fee_pool.addr().clone())
        .unwrap();
    assert_eq!(fee_pool_balance, Uint128::from(15_000_000_000u128));
}

#[test]
fn test_force_error_insufficient_balance_open_position_total_fee_ten_percent() {
    let SimpleScenario {
        mut router,
        owner,
        alice,
        usdc,
        engine,
        vamm,
        ..
    } = new_simple_scenario();

    // 10% fee
    let msg = vamm.set_toll_ratio(Uint128::from(50_000_000u128)).unwrap();
    router.execute(owner.clone(), msg).unwrap();

    let msg = vamm
        .set_spread_ratio(Uint128::from(50_000_000u128))
        .unwrap();
    router.execute(owner.clone(), msg).unwrap();

    // reduce the allowance to 359
    router
        .execute_contract(
            alice.clone(),
            usdc.addr().clone(),
            &Cw20ExecuteMsg::DecreaseAllowance {
                spender: engine.addr().to_string(),
                amount: Uint128::from(1_641_000_000_000u64),
                expires: None,
            },
            &[],
        )
        .unwrap();

    let msg = engine
        .open_position(
            vamm.addr().to_string(),
            Side::Buy,
            Uint128::from(400_000_000_000u64),
            Uint128::from(2_000_000_000u64),
            Some(to_decimals(28)),
            Some(to_decimals(8)),
            Uint128::from(17_500_000_000u64),
            vec![],
        )
        .unwrap();
    let err = router.execute(alice.clone(), msg).unwrap_err();
    assert_eq!(
        StdError::GenericErr {
            msg: format!(
                "transfer failure - reply (id {})",
                TRANSFER_FAILURE_REPLY_ID
            ),
        },
        err.downcast().unwrap()
    );
}

#[test]
fn test_has_spread_no_toll() {
    let SimpleScenario {
        mut router,
        owner,
        alice,
        usdc,
        fee_pool,
        engine,
        vamm,
        insurance_fund,
        ..
    } = new_simple_scenario();

    // 10% fee
    let msg = vamm.set_toll_ratio(Uint128::zero()).unwrap();
    router.execute(owner.clone(), msg).unwrap();

    let msg = vamm
        .set_spread_ratio(Uint128::from(100_000_000u128))
        .unwrap();
    router.execute(owner.clone(), msg).unwrap();

    let msg = engine
        .open_position(
            vamm.addr().to_string(),
            Side::Buy,
            Uint128::from(300_000_000_000u64),
            Uint128::from(2_000_000_000u64),
            Some(to_decimals(18)),
            Some(to_decimals(8)),
            Uint128::zero(),
            vec![],
        )
        .unwrap();
    router.execute(alice.clone(), msg).unwrap();

    let fee_pool_balance = usdc
        .balance(&router.wrap(), fee_pool.addr().clone())
        .unwrap();
    assert_eq!(fee_pool_balance, Uint128::zero());

    let insurance_balance = usdc
        .balance(&router.wrap(), insurance_fund.addr().clone())
        .unwrap();
    assert_eq!(insurance_balance, Uint128::from(5060_000_000_000u128));
}
