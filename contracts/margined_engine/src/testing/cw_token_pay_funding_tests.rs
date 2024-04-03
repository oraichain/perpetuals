use cosmwasm_std::Uint128;
use margined_common::integer::Integer;
use margined_perp::margined_engine::Side;
use margined_utils::{
    cw_multi_test::Executor,
    testing::{to_decimals, SimpleScenario},
};

use crate::testing::new_simple_scenario;

pub const NEXT_FUNDING_PERIOD_DELTA: u64 = 86_400u64;

#[test]
fn test_generate_loss_for_amm_when_funding_rate_is_positive_and_amm_is_long() {
    let SimpleScenario {
        mut router,
        alice,
        bob,
        owner,
        insurance_fund,
        engine,
        vamm,
        usdc,
        pricefeed,
        ..
    } = new_simple_scenario();

    let msg = engine
        .open_position(
            vamm.addr().to_string(),
            Side::Buy,
            to_decimals(300u64),
            to_decimals(2u64),
            Some(to_decimals(17)),
            Some(Uint128::zero()),
            to_decimals(0u64),
            vec![],
        )
        .unwrap();
    router.execute(alice.clone(), msg).unwrap();

    let msg = engine
        .open_position(
            vamm.addr().to_string(),
            Side::Sell,
            to_decimals(1200u64),
            to_decimals(1u64),
            Some(to_decimals(5)),
            Some(to_decimals(28)),
            to_decimals(0u64),
            vec![],
        )
        .unwrap();
    router.execute(bob.clone(), msg).unwrap();

    let engine_balance = usdc.balance(&router.wrap(), engine.addr().clone()).unwrap();
    assert_eq!(engine_balance, Uint128::from(1_500_000_000_000u128));

    let price = Uint128::from(1_590_000_000u128);
    let timestamp = 1_000_000_000;

    let msg = pricefeed
        .append_price("ETH".to_string(), price, timestamp)
        .unwrap();
    router.execute(owner.clone(), msg).unwrap();

    // move to the next funding time
    router.update_block(|block| {
        block.time = block.time.plus_seconds(NEXT_FUNDING_PERIOD_DELTA);
        block.height += 1;
    });

    let msg = engine.pay_funding(vamm.addr().to_string()).unwrap();
    router.execute(owner.clone(), msg).unwrap();

    let premium_fraction = engine
        .get_latest_cumulative_premium_fraction(&router.wrap(), vamm.addr().to_string())
        .unwrap();
    assert_eq!(
        premium_fraction,
        Integer::new_positive(10_000_000u128), // 0.01
    );

    // then alice need to pay 1% of her position size as fundingPayment
    // {balance: 37.5, margin: 300} => {balance: 37.5, margin: 299.625}
    let alice_position = engine
        .get_position_with_funding_payment(&router.wrap(), vamm.addr().to_string(), 1)
        .unwrap();
    assert_eq!(
        alice_position.size,
        Integer::new_positive(37_500_000_000u128)
    );
    assert_eq!(alice_position.margin, Uint128::from(299_625_000_000u128));

    // then bob will get 1% of his position size as fundingPayment
    // {balance: -187.5, margin: 1200} => {balance: -187.5, margin: 1201.875}
    let bob_position = engine
        .get_position_with_funding_payment(&router.wrap(), vamm.addr().to_string(), 2)
        .unwrap();
    assert_eq!(
        bob_position.size,
        Integer::new_negative(187_500_000_000u128)
    );
    assert_eq!(bob_position.margin, Uint128::from(1_201_875_000_000u128));

    // then fundingPayment will generate 1.5 loss and clearingHouse will withdraw in advanced from insuranceFund
    // clearingHouse: 1500 + 1.5
    // insuranceFund: 5000 - 1.5
    let engine_balance = usdc.balance(&router.wrap(), engine.addr().clone()).unwrap();
    assert_eq!(engine_balance, Uint128::from(1_501_500_000_000u128));
    let insurance_balance = usdc.balance(&router.wrap(), insurance_fund.addr()).unwrap();
    assert_eq!(insurance_balance, Uint128::from(4_998_500_000_000u128));
}

#[test]
fn test_will_keep_generating_same_loss_when_funding_rate_is_positive() {
    let SimpleScenario {
        mut router,
        alice,
        bob,
        owner,
        insurance_fund,
        engine,
        vamm,
        usdc,
        pricefeed,
        ..
    } = new_simple_scenario();

    let msg = engine
        .open_position(
            vamm.addr().to_string(),
            Side::Buy,
            to_decimals(300u64),
            to_decimals(2u64),
            Some(to_decimals(18)),
            Some(Uint128::zero()),
            to_decimals(0u64),
            vec![],
        )
        .unwrap();
    router.execute(alice.clone(), msg).unwrap();

    let msg = engine
        .open_position(
            vamm.addr().to_string(),
            Side::Sell,
            to_decimals(1200u64),
            to_decimals(1u64),
            Some(to_decimals(5)),
            Some(to_decimals(26)),
            to_decimals(0u64),
            vec![],
        )
        .unwrap();
    router.execute(bob.clone(), msg).unwrap();

    let price = Uint128::from(1_590_000_000u128);
    let timestamp = 1_000_000_000;

    let msg = pricefeed
        .append_price("ETH".to_string(), price, timestamp)
        .unwrap();
    router.execute(owner.clone(), msg).unwrap();

    // move to the next funding time
    router.update_block(|block| {
        block.time = block.time.plus_seconds(NEXT_FUNDING_PERIOD_DELTA);
        block.height += 1;
    });

    let msg = engine.pay_funding(vamm.addr().to_string()).unwrap();
    router.execute(owner.clone(), msg).unwrap();

    // move to the next funding time
    router.update_block(|block| {
        block.time = block.time.plus_seconds(NEXT_FUNDING_PERIOD_DELTA);
        block.height += 1;
    });

    let msg = engine.pay_funding(vamm.addr().to_string()).unwrap();
    router.execute(owner.clone(), msg).unwrap();

    // same as above test case:
    // there are only 2 traders: bob and alice
    // alice need to pay 1% of her position size as fundingPayment (37.5 * 1% = 0.375)
    // bob will get 1% of his position size as fundingPayment (187.5 * 1% = 1.875)
    // ammPnl = 0.375 - 1.875 = -1.5
    // clearingHouse payFunding twice in the same condition
    // then fundingPayment will generate 1.5 * 2 loss and clearingHouse will withdraw in advanced from insuranceFund
    // clearingHouse: 1500 + 3
    // insuranceFund: 5000 - 3
    let engine_balance = usdc.balance(&router.wrap(), engine.addr().clone()).unwrap();
    assert_eq!(engine_balance, Uint128::from(1_503_000_000_000u128));
    let insurance_balance = usdc.balance(&router.wrap(), insurance_fund.addr()).unwrap();
    assert_eq!(insurance_balance, Uint128::from(4_997_000_000_000u128));
}

#[test]
fn test_funding_rate_is_1_percent_then_negative_1_percent() {
    let SimpleScenario {
        mut router,
        alice,
        bob,
        owner,
        engine,
        vamm,
        pricefeed,
        ..
    } = new_simple_scenario();

    let msg = engine
        .open_position(
            vamm.addr().to_string(),
            Side::Buy,
            to_decimals(300u64),
            to_decimals(2u64),
            Some(to_decimals(17)),
            Some(Uint128::zero()),
            to_decimals(0u64),
            vec![],
        )
        .unwrap();
    router.execute(alice.clone(), msg).unwrap();

    let msg = engine
        .open_position(
            vamm.addr().to_string(),
            Side::Sell,
            to_decimals(1200u64),
            to_decimals(1u64),
            Some(to_decimals(5)),
            Some(to_decimals(26)),
            to_decimals(0u64),
            vec![],
        )
        .unwrap();
    router.execute(bob.clone(), msg).unwrap();

    let price = Uint128::from(1_590_000_000u128);
    let timestamp = 1_000_000_000;

    let msg = pricefeed
        .append_price("ETH".to_string(), price, timestamp)
        .unwrap();
    router.execute(owner.clone(), msg).unwrap();

    // move to the next funding time
    router.update_block(|block| {
        block.time = block.time.plus_seconds(NEXT_FUNDING_PERIOD_DELTA);
        block.height += 1;
    });

    let msg = engine.pay_funding(vamm.addr().to_string()).unwrap();
    router.execute(owner.clone(), msg).unwrap();

    let premium_fraction = engine
        .get_latest_cumulative_premium_fraction(&router.wrap(), vamm.addr().to_string())
        .unwrap();
    assert_eq!(
        premium_fraction,
        Integer::new_positive(10_000_000u128), // 0.01
    );

    // then alice need to pay 1% of her position size as fundingPayment
    // {balance: 37.5, margin: 300} => {balance: 37.5, margin: 299.625}
    let alice_position = engine
        .get_position_with_funding_payment(&router.wrap(), vamm.addr().to_string(), 1)
        .unwrap();
    assert_eq!(alice_position.margin, Uint128::from(299_625_000_000u128));
    let alice_balance = engine
        .get_balance_with_funding_payment(&router.wrap(), 1)
        .unwrap();
    assert_eq!(alice_balance, Uint128::from(299_625_000_000u128));

    // move to the next funding time
    router.update_block(|block| {
        block.time = block.time.plus_seconds(NEXT_FUNDING_PERIOD_DELTA);
        block.height += 1;
    });

    let msg = engine.pay_funding(vamm.addr().to_string()).unwrap();
    router.execute(owner.clone(), msg).unwrap();

    let alice_position = engine
        .get_position_with_funding_payment(&router.wrap(), vamm.addr().to_string(), 1)
        .unwrap();
    assert_eq!(alice_position.margin, Uint128::from(299_250_000_000u128));
    let alice_balance = engine
        .get_balance_with_funding_payment(&router.wrap(), 1)
        .unwrap();
    assert_eq!(alice_balance, Uint128::from(299_250_000_000u128));

    let price = Uint128::from(1_610_000_000u128);
    let timestamp = 1_000_000_000;

    let msg = pricefeed
        .append_price("ETH".to_string(), price, timestamp)
        .unwrap();
    router.execute(owner.clone(), msg).unwrap();

    // move to the next funding time
    router.update_block(|block| {
        block.time = block.time.plus_seconds(NEXT_FUNDING_PERIOD_DELTA);
        block.height += 1;
    });

    let msg = engine.pay_funding(vamm.addr().to_string()).unwrap();
    router.execute(owner.clone(), msg).unwrap();

    let alice_position = engine
        .get_position_with_funding_payment(&router.wrap(), vamm.addr().to_string(), 1)
        .unwrap();
    assert_eq!(alice_position.margin, Uint128::from(299_625_000_000u128));
    let alice_balance = engine
        .get_balance_with_funding_payment(&router.wrap(), 1)
        .unwrap();
    assert_eq!(alice_balance, Uint128::from(299_625_000_000u128));
}

#[test]
fn test_funding_rate_is_negative_1_percent_then_negative_1_percent() {
    let SimpleScenario {
        mut router,
        alice,
        bob,
        owner,
        engine,
        vamm,
        pricefeed,
        ..
    } = new_simple_scenario();

    let msg = engine
        .open_position(
            vamm.addr().to_string(),
            Side::Buy,
            to_decimals(300u64),
            to_decimals(2u64),
            Some(to_decimals(18)),
            Some(Uint128::zero()),
            to_decimals(0u64),
            vec![],
        )
        .unwrap();
    router.execute(alice.clone(), msg).unwrap();

    let msg = engine
        .open_position(
            vamm.addr().to_string(),
            Side::Sell,
            to_decimals(1200u64),
            to_decimals(1u64),
            Some(to_decimals(5)),
            Some(to_decimals(26)),
            to_decimals(0u64),
            vec![],
        )
        .unwrap();
    router.execute(bob.clone(), msg).unwrap();

    let price = Uint128::from(1_590_000_000u128);
    let timestamp = 1_000_000_000;

    let msg = pricefeed
        .append_price("ETH".to_string(), price, timestamp)
        .unwrap();
    router.execute(owner.clone(), msg).unwrap();

    // move to the next funding time
    router.update_block(|block| {
        block.time = block.time.plus_seconds(NEXT_FUNDING_PERIOD_DELTA);
        block.height += 1;
    });

    let msg = engine.pay_funding(vamm.addr().to_string()).unwrap();
    router.execute(owner.clone(), msg).unwrap();

    let premium_fraction = engine
        .get_latest_cumulative_premium_fraction(&router.wrap(), vamm.addr().to_string())
        .unwrap();
    assert_eq!(
        premium_fraction,
        Integer::new_positive(10_000_000u128), // 0.01
    );

    // then alice need to pay 1% of her position size as fundingPayment
    // {size: 37.5, margin: 300} => {size: 37.5, margin: 299.625}
    let alice_position = engine
        .get_position_with_funding_payment(&router.wrap(), vamm.addr().to_string(), 1)
        .unwrap();
    assert_eq!(alice_position.margin, Uint128::from(299_625_000_000u128));
    let alice_balance = engine
        .get_balance_with_funding_payment(&router.wrap(), 1)
        .unwrap();
    assert_eq!(alice_balance, Uint128::from(299_625_000_000u128));

    let price = Uint128::from(1_610_000_000u128);
    let timestamp = 1_000_000_000;

    let msg = pricefeed
        .append_price("ETH".to_string(), price, timestamp)
        .unwrap();
    router.execute(owner.clone(), msg).unwrap();

    // move to the next funding time
    router.update_block(|block| {
        block.time = block.time.plus_seconds(NEXT_FUNDING_PERIOD_DELTA);
        block.height += 1;
    });

    let msg = engine.pay_funding(vamm.addr().to_string()).unwrap();
    router.execute(owner.clone(), msg).unwrap();

    let premium_fraction = engine
        .get_latest_cumulative_premium_fraction(&router.wrap(), vamm.addr().to_string())
        .unwrap();
    assert_eq!(premium_fraction, Integer::zero());

    let alice_position = engine
        .get_position_with_funding_payment(&router.wrap(), vamm.addr().to_string(), 1)
        .unwrap();
    assert_eq!(alice_position.margin, Uint128::from(300_000_000_000u128));
    let alice_balance = engine
        .get_balance_with_funding_payment(&router.wrap(), 1)
        .unwrap();
    assert_eq!(alice_balance, Uint128::from(300_000_000_000u128));

    // move to the next funding time
    router.update_block(|block| {
        block.time = block.time.plus_seconds(NEXT_FUNDING_PERIOD_DELTA);
        block.height += 1;
    });

    let msg = engine.pay_funding(vamm.addr().to_string()).unwrap();
    router.execute(owner.clone(), msg).unwrap();

    let premium_fraction = engine
        .get_latest_cumulative_premium_fraction(&router.wrap(), vamm.addr().to_string())
        .unwrap();
    assert_eq!(
        premium_fraction,
        Integer::new_negative(10_000_000u128), // 0.01
    );

    let alice_position = engine
        .get_position_with_funding_payment(&router.wrap(), vamm.addr().to_string(), 1)
        .unwrap();
    assert_eq!(alice_position.margin, Uint128::from(300_375_000_000u128));
    let alice_balance = engine
        .get_balance_with_funding_payment(&router.wrap(), 1)
        .unwrap();
    assert_eq!(alice_balance, Uint128::from(300_375_000_000u128));
}

#[test]
fn test_have_huge_funding_payment_profit_withdraw_excess_margin() {
    let SimpleScenario {
        mut router,
        alice,
        bob,
        owner,
        engine,
        vamm,
        pricefeed,
        ..
    } = new_simple_scenario();

    let msg = engine
        .open_position(
            vamm.addr().to_string(),
            Side::Buy,
            to_decimals(300u64),
            to_decimals(2u64),
            Some(to_decimals(18)),
            Some(Uint128::zero()),
            to_decimals(0u64),
            vec![],
        )
        .unwrap();
    router.execute(alice.clone(), msg).unwrap();

    let msg = engine
        .open_position(
            vamm.addr().to_string(),
            Side::Sell,
            to_decimals(1200u64),
            to_decimals(1u64),
            Some(to_decimals(5)),
            Some(to_decimals(26)),
            to_decimals(0u64),
            vec![],
        )
        .unwrap();
    router.execute(bob.clone(), msg).unwrap();

    let price = Uint128::from(21_600_000_000u128);
    let timestamp = 1_000_000_000;

    let msg = pricefeed
        .append_price("ETH".to_string(), price, timestamp)
        .unwrap();
    router.execute(owner.clone(), msg).unwrap();

    // move to the next funding time
    router.update_block(|block| {
        block.time = block.time.plus_seconds(NEXT_FUNDING_PERIOD_DELTA);
        block.height += 1;
    });

    let msg = engine.pay_funding(vamm.addr().to_string()).unwrap();
    router.execute(owner.clone(), msg).unwrap();

    // margin = 1050 - 400 = 650
    let alice_position = engine
        .get_position_with_funding_payment(&router.wrap(), vamm.addr().to_string(), 1)
        .unwrap();
    assert_eq!(alice_position.margin, to_decimals(1050u64));

    // then alice will get 2000% of her position size as fundingPayment
    // {balance: 37.5, margin: 300} => {balance: 37.5, margin: 1050}
    // then alice can withdraw more than her initial margin while remain the enough margin ratio
    let msg = engine
        .withdraw_margin(vamm.addr().to_string(), 1, to_decimals(400u64))
        .unwrap();
    router.execute(alice.clone(), msg).unwrap();

    // margin = 1050 - 400 = 650
    let alice_position = engine
        .get_position_with_funding_payment(&router.wrap(), vamm.addr().to_string(), 1)
        .unwrap();
    assert_eq!(alice_position.margin, to_decimals(650u64));
    let alice_balance = engine
        .get_balance_with_funding_payment(&router.wrap(), 1)
        .unwrap();
    assert_eq!(alice_balance, to_decimals(650u64));
}

#[test]
fn test_have_huge_funding_payment_margin_zero_with_bad_debt() {
    let SimpleScenario {
        mut router,
        alice,
        bob,
        owner,
        engine,
        vamm,
        pricefeed,
        ..
    } = new_simple_scenario();

    let msg = engine
        .open_position(
            vamm.addr().to_string(),
            Side::Buy,
            to_decimals(300u64),
            to_decimals(2u64),
            Some(to_decimals(18)),
            Some(Uint128::zero()),
            to_decimals(0u64),
            vec![],
        )
        .unwrap();
    router.execute(alice.clone(), msg).unwrap();

    let msg = engine
        .open_position(
            vamm.addr().to_string(),
            Side::Sell,
            to_decimals(1200u64),
            to_decimals(1u64),
            Some(to_decimals(5)),
            Some(to_decimals(26)),
            to_decimals(0u64),
            vec![],
        )
        .unwrap();
    router.execute(bob.clone(), msg).unwrap();

    let price = Uint128::from(21_600_000_000u128);
    let timestamp = 1_000_000_000;

    let msg = pricefeed
        .append_price("ETH".to_string(), price, timestamp)
        .unwrap();
    router.execute(owner.clone(), msg).unwrap();

    // move to the next funding time
    router.update_block(|block| {
        block.time = block.time.plus_seconds(NEXT_FUNDING_PERIOD_DELTA);
        block.height += 1;
    });

    let msg = engine.pay_funding(vamm.addr().to_string()).unwrap();
    router.execute(owner.clone(), msg).unwrap();

    // then bob will get 2000% of his position size as fundingPayment
    // funding payment: -187.5 x 2000% = -3750, margin is 1200 so bad debt = -3750 + 1200 = 2550
    let bob_position = engine
        .get_position_with_funding_payment(&router.wrap(), vamm.addr().to_string(), 2)
        .unwrap();
    assert_eq!(bob_position.margin, to_decimals(0u64));

    let msg = engine
        .liquidate(vamm.addr().to_string(), 2, to_decimals(0u64))
        .unwrap();

    let response = router.execute(bob.clone(), msg).unwrap();
    assert_eq!(
        response.events[5].attributes[6].value,
        Uint128::from(3_750_000_000_000u128).to_string()
    ); // funding payment
    assert_eq!(
        response.events[5].attributes[7].value,
        Uint128::from(2_580_000_000_000u128).to_string()
    ); // bad debt
}

#[test]
fn test_have_huge_funding_payment_margin_zero_can_add_margin() {
    let SimpleScenario {
        mut router,
        alice,
        bob,
        owner,
        engine,
        vamm,
        pricefeed,
        usdc,
        ..
    } = new_simple_scenario();

    let msg = engine
        .open_position(
            vamm.addr().to_string(),
            Side::Buy,
            to_decimals(300u64),
            to_decimals(2u64),
            Some(to_decimals(18)),
            Some(Uint128::zero()),
            to_decimals(0u64),
            vec![],
        )
        .unwrap();
    router.execute(alice.clone(), msg).unwrap();

    let msg = engine
        .open_position(
            vamm.addr().to_string(),
            Side::Sell,
            to_decimals(1200u64),
            to_decimals(1u64),
            Some(to_decimals(5)),
            Some(to_decimals(26)),
            to_decimals(0u64),
            vec![],
        )
        .unwrap();
    router.execute(bob.clone(), msg).unwrap();

    let price = Uint128::from(21_600_000_000u128);
    let timestamp = 1_000_000_000;

    let msg = pricefeed
        .append_price("ETH".to_string(), price, timestamp)
        .unwrap();
    router.execute(owner.clone(), msg).unwrap();

    // move to the next funding time
    router.update_block(|block| {
        block.time = block.time.plus_seconds(NEXT_FUNDING_PERIOD_DELTA);
        block.height += 1;
    });

    let msg = engine.pay_funding(vamm.addr().to_string()).unwrap();
    router.execute(owner.clone(), msg).unwrap();

    let engine_balance = usdc.balance(&router.wrap(), engine.addr().clone()).unwrap();
    assert_eq!(engine_balance, Uint128::from(0u128));

    // then bob will get 2000% of his position size as fundingPayment
    // funding payment: -187.5 x 2000% = -3750, margin is 1200 so bad debt = -3750 + 1200 = 2550
    // margin can be added but will still shows 0 until it's larger than bad debt
    let msg = engine
        .deposit_margin(vamm.addr().to_string(), 2, to_decimals(1u64), vec![])
        .unwrap();
    router.execute(bob.clone(), msg).unwrap();

    let bob_position = engine
        .get_position_with_funding_payment(&router.wrap(), vamm.addr().to_string(), 2)
        .unwrap();
    assert_eq!(bob_position.margin, to_decimals(0u64));

    let engine_balance = usdc.balance(&router.wrap(), engine.addr().clone()).unwrap();
    assert_eq!(engine_balance, to_decimals(1u64));
}

#[test]
fn test_have_huge_funding_payment_loss_margin_zero_cannot_remove_margin() {
    let SimpleScenario {
        mut router,
        alice,
        bob,
        owner,
        engine,
        vamm,
        pricefeed,
        usdc,
        ..
    } = new_simple_scenario();

    let msg = engine
        .open_position(
            vamm.addr().to_string(),
            Side::Buy,
            to_decimals(300u64),
            to_decimals(2u64),
            Some(to_decimals(18)),
            Some(Uint128::zero()),
            to_decimals(0u64),
            vec![],
        )
        .unwrap();
    router.execute(alice.clone(), msg).unwrap();

    let msg = engine
        .open_position(
            vamm.addr().to_string(),
            Side::Sell,
            to_decimals(1200u64),
            to_decimals(1u64),
            Some(to_decimals(5)),
            Some(to_decimals(26)),
            to_decimals(0u64),
            vec![],
        )
        .unwrap();
    router.execute(bob.clone(), msg).unwrap();

    let price = Uint128::from(21_600_000_000u128);
    let timestamp = 1_000_000_000;

    let msg = pricefeed
        .append_price("ETH".to_string(), price, timestamp)
        .unwrap();
    router.execute(owner.clone(), msg).unwrap();

    // move to the next funding time
    router.update_block(|block| {
        block.time = block.time.plus_seconds(NEXT_FUNDING_PERIOD_DELTA);
        block.height += 1;
    });

    let msg = engine.pay_funding(vamm.addr().to_string()).unwrap();
    router.execute(owner.clone(), msg).unwrap();

    let engine_balance = usdc.balance(&router.wrap(), engine.addr().clone()).unwrap();
    assert_eq!(engine_balance, Uint128::from(0u128));

    // then bob will get 2000% of his position size as fundingPayment
    // funding payment: -187.5 x 2000% = -3750, margin is 1200 so bad debt = -3750 + 1200 = 2550
    // margin can't removed
    let msg = engine
        .withdraw_margin(vamm.addr().to_string(), 2, Uint128::from(1u64))
        .unwrap();
    let err = router.execute(bob.clone(), msg).unwrap_err();
    assert_eq!(
        err.source().unwrap().to_string(),
        "Generic error: Insufficient margin".to_string()
    );
}

#[test]
fn test_reduce_bad_debt_after_adding_margin_to_an_underwater_position() {
    let SimpleScenario {
        mut router,
        alice,
        bob,
        owner,
        engine,
        vamm,
        pricefeed,
        usdc,
        ..
    } = new_simple_scenario();

    let msg = engine
        .open_position(
            vamm.addr().to_string(),
            Side::Buy,
            to_decimals(300u64),
            to_decimals(2u64),
            Some(to_decimals(18)),
            Some(Uint128::zero()),
            to_decimals(0u64),
            vec![],
        )
        .unwrap();
    router.execute(alice.clone(), msg).unwrap();

    let msg = engine
        .open_position(
            vamm.addr().to_string(),
            Side::Sell,
            to_decimals(1200u64),
            to_decimals(1u64),
            Some(to_decimals(5)),
            Some(to_decimals(26)),
            to_decimals(0u64),
            vec![],
        )
        .unwrap();
    router.execute(bob.clone(), msg).unwrap();

    let price = Uint128::from(21_600_000_000u128);
    let timestamp = 1_000_000_000;

    let msg = pricefeed
        .append_price("ETH".to_string(), price, timestamp)
        .unwrap();
    router.execute(owner.clone(), msg).unwrap();

    // move to the next funding time
    router.update_block(|block| {
        block.time = block.time.plus_seconds(NEXT_FUNDING_PERIOD_DELTA);
        block.height += 1;
    });

    let msg = engine.pay_funding(vamm.addr().to_string()).unwrap();
    router.execute(owner.clone(), msg).unwrap();

    let engine_balance = usdc.balance(&router.wrap(), engine.addr().clone()).unwrap();
    assert_eq!(engine_balance, Uint128::from(0u128));

    // then bob will get 2000% of his position size as fundingPayment
    // funding payment: -187.5 x 2000% = -3750, margin is 1200 so bad debt = -3750 + 1200 = 2550
    // margin can be added but will still shows 0 until it's larger than bad debt
    // margin can't removed
    let msg = engine
        .deposit_margin(vamm.addr().to_string(), 2, to_decimals(10u64), vec![])
        .unwrap();
    router.execute(bob.clone(), msg).unwrap();

    // badDebt 2550 - 10 margin = 2540
    let msg = engine
        .liquidate(vamm.addr().to_string(), 2, Uint128::zero())
        .unwrap();
    let response = router.execute(bob.clone(), msg).unwrap();
    assert_eq!(
        response.events[5].attributes[6].value,
        Uint128::from(3_750_000_000_000u128).to_string()
    ); // funding payment
    assert_eq!(
        response.events[5].attributes[7].value,
        Uint128::from(2_570_000_000_000u128).to_string()
    ); // bad debt
}

#[test]
fn test_will_change_nothing_if_funding_rate_is_zero() {
    let SimpleScenario {
        mut router,
        alice,
        bob,
        owner,
        insurance_fund,
        engine,
        vamm,
        pricefeed,
        usdc,
        ..
    } = new_simple_scenario();

    let msg = engine
        .open_position(
            vamm.addr().to_string(),
            Side::Buy,
            to_decimals(300u64),
            to_decimals(2u64),
            Some(to_decimals(18)),
            Some(Uint128::zero()),
            to_decimals(0u64),
            vec![],
        )
        .unwrap();
    router.execute(alice.clone(), msg).unwrap();

    let msg = engine
        .open_position(
            vamm.addr().to_string(),
            Side::Sell,
            to_decimals(1200u64),
            to_decimals(1u64),
            Some(to_decimals(5)),
            Some(to_decimals(26)),
            to_decimals(0u64),
            vec![],
        )
        .unwrap();
    router.execute(bob.clone(), msg).unwrap();

    let price = Uint128::from(1_600_000_000u128);
    let timestamp = 1_000_000_000;

    let msg = pricefeed
        .append_price("ETH".to_string(), price, timestamp)
        .unwrap();
    router.execute(owner.clone(), msg).unwrap();

    // move to the next funding time
    router.update_block(|block| {
        block.time = block.time.plus_seconds(NEXT_FUNDING_PERIOD_DELTA);
        block.height += 1;
    });

    let msg = engine.pay_funding(vamm.addr().to_string()).unwrap();
    router.execute(owner.clone(), msg).unwrap();

    let premium_fraction = engine
        .get_latest_cumulative_premium_fraction(&router.wrap(), vamm.addr().to_string())
        .unwrap();
    assert_eq!(
        premium_fraction,
        Integer::zero(), // 0.00
    );

    // then alice's position won't change
    // {balance: 37.5, margin: 300}
    let alice_position = engine
        .get_position_with_funding_payment(&router.wrap(), vamm.addr().to_string(), 1)
        .unwrap();
    assert_eq!(
        alice_position.size,
        Integer::new_positive(37_500_000_000u128)
    );
    assert_eq!(alice_position.margin, Uint128::from(300_000_000_000u128));

    // then bob's position won't change
    // {balance: -187.5, margin: 1200}
    let bob_position = engine
        .get_position_with_funding_payment(&router.wrap(), vamm.addr().to_string(), 2)
        .unwrap();
    assert_eq!(
        bob_position.size,
        Integer::new_negative(187_500_000_000u128)
    );
    assert_eq!(bob_position.margin, Uint128::from(1_200_000_000_000u128));

    // clearingHouse: 1500
    // insuranceFund: 5000
    let engine_balance = usdc.balance(&router.wrap(), engine.addr().clone()).unwrap();
    assert_eq!(engine_balance, Uint128::from(1_500_000_000_000u128));
    let insurance_balance = usdc.balance(&router.wrap(), insurance_fund.addr()).unwrap();
    assert_eq!(insurance_balance, Uint128::from(5_000_000_000_000u128));
}
