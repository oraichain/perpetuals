use cosmwasm_std::{BankMsg, Coin, CosmosMsg, StdError, Uint128};
use margined_common::integer::Integer;
use margined_perp::margined_engine::{PnlCalcOption, Side};
use margined_utils::{
    cw_multi_test::Executor,
    testing::{to_decimals, NativeTokenScenario},
};

use crate::{contract::INCREASE_POSITION_REPLY_ID, testing::new_native_token_scenario};

#[test]
fn test_partially_liquidate_long_position() {
    let NativeTokenScenario {
        mut router,
        alice,
        bob,
        carol,
        owner,
        insurance_fund,
        engine,
        vamm,
        pricefeed,
        ..
    } = new_native_token_scenario();

    // set the latest price
    let price = Uint128::from(10_000_000u128);
    let timestamp = router.block_info().time.seconds();

    let msg = pricefeed
        .append_price("ETH".to_string(), price, timestamp)
        .unwrap();
    router.execute(owner.clone(), msg).unwrap();

    router.update_block(|block| {
        block.time = block.time.plus_seconds(900);
        block.height += 1;
    });

    let msg = engine
        .set_margin_ratios(Uint128::from(100_000u128))
        .unwrap();
    router.execute(owner.clone(), msg).unwrap();

    let msg = engine
        .set_partial_liquidation_ratio(Uint128::from(250_000u128))
        .unwrap();
    router.execute(owner.clone(), msg).unwrap();

    let msg = engine
        .set_liquidation_fee(Uint128::from(25_000u128))
        .unwrap();
    router.execute(owner.clone(), msg).unwrap();

    // when alice create a 25 margin * 10x position to get 20 long position
    // AMM after: 1250 : 80
    let msg = engine
        .open_position(
            vamm.addr().to_string(),
            Side::Buy,
            Uint128::from(25_000_000u64),
            Uint128::from(10_000_000u64),
            Some(to_decimals(10)),
            Some(Uint128::zero()),
            Uint128::zero(),
            vec![Coin::new(25_000_000u128, "orai")],
        )
        .unwrap();
    router.execute(alice.clone(), msg).unwrap();

    router.update_block(|block| {
        block.time = block.time.plus_seconds(15);
        block.height += 1;
    });

    // when bob create a 45.18072289 margin * 1x position to get 3 short position
    // AMM after: 1204.819277 : 83
    let msg = engine
        .open_position(
            vamm.addr().to_string(),
            Side::Sell,
            Uint128::from(45_180_723u128),
            Uint128::from(1_000_000u64),
            Some(Uint128::from(6_000_000u64)),
            Some(Uint128::from(18_000_000u64)),
            Uint128::zero(),
            vec![Coin::new(45_180_723u128, "orai")],
        )
        .unwrap();
    router.execute(bob.clone(), msg).unwrap();

    let msg = engine
        .liquidate(vamm.addr().to_string(), 1, Uint128::zero())
        .unwrap();
    router.execute(carol.clone(), msg).unwrap();

    let position = engine
        .position(&router.wrap(), vamm.addr().to_string(), 1)
        .unwrap();
    assert_eq!(position.margin, Uint128::from(19_274_982u128));
    assert_eq!(position.size, Integer::new_positive(15_000_000u128));

    let margin_ratio = engine
        .get_margin_ratio(&router.wrap(), vamm.addr().to_string(), 1)
        .unwrap();
    assert_eq!(margin_ratio, Integer::new_positive(43_713u128));

    let carol_balance = router.wrap().query_balance(&carol, "orai").unwrap().amount;
    assert_eq!(carol_balance, Uint128::from(855_695u128));

    let insurance_balance = router
        .wrap()
        .query_balance(&insurance_fund.addr(), "orai")
        .unwrap()
        .amount;
    assert_eq!(insurance_balance, Uint128::from(5_000_855_695u128));
}

#[test]
fn test_partially_liquidate_long_position_with_quote_asset_limit() {
    let NativeTokenScenario {
        mut router,
        alice,
        bob,
        carol,
        owner,
        engine,
        vamm,
        pricefeed,
        ..
    } = new_native_token_scenario();

    // set the latest price
    let price = Uint128::from(10_000_000u128);
    let timestamp = router.block_info().time.seconds();

    let msg = pricefeed
        .append_price("ETH".to_string(), price, timestamp)
        .unwrap();
    router.execute(owner.clone(), msg).unwrap();

    router.update_block(|block| {
        block.time = block.time.plus_seconds(900);
        block.height += 1;
    });

    let msg = engine
        .set_margin_ratios(Uint128::from(100_000u128))
        .unwrap();
    router.execute(owner.clone(), msg).unwrap();

    let msg = engine
        .set_partial_liquidation_ratio(Uint128::from(250_000u128))
        .unwrap();
    router.execute(owner.clone(), msg).unwrap();

    let msg = engine
        .set_liquidation_fee(Uint128::from(25_000u128))
        .unwrap();
    router.execute(owner.clone(), msg).unwrap();

    // when alice create a 25 margin * 10x position to get 20 long position
    // AMM after: 1250 : 80
    let msg = engine
        .open_position(
            vamm.addr().to_string(),
            Side::Buy,
            Uint128::from(25_000_000u64),
            Uint128::from(10_000_000u64),
            Some(Uint128::from(13_000_000u64)),
            Some(Uint128::from(9_000_000u64)),
            Uint128::zero(),
            vec![Coin::new(25_000_000u128, "orai")],
        )
        .unwrap();
    router.execute(alice.clone(), msg).unwrap();

    router.update_block(|block| {
        block.time = block.time.plus_seconds(15);
        block.height += 1;
    });

    // when bob create a 45.18072289 margin * 1x position to get 3 short position
    // AMM after: 1204.819277 : 83
    let msg = engine
        .open_position(
            vamm.addr().to_string(),
            Side::Sell,
            Uint128::from(45_180_723u128),
            Uint128::from(1_000_000u64),
            Some(Uint128::from(6_000_000u64)),
            Some(Uint128::from(18_000_000u64)),
            Uint128::zero(),
            vec![Coin::new(45_180_723u128, "orai")],
        )
        .unwrap();
    router.execute(bob.clone(), msg).unwrap();

    // partially liquidate 25%
    // liquidated positionNotional: getOutputPrice(20 (original position) * 0.25) = 68.455
    // if quoteAssetAmountLimit == 273.85 > 68.455 * 4 = 273.82, quote asset gets is less than expected, thus tx reverts
    let msg = engine
        .liquidate(vamm.addr().to_string(), 1, Uint128::from(273_850_000u64))
        .unwrap();
    let err = router.execute(carol.clone(), msg).unwrap_err();
    assert_eq!(
        StdError::GenericErr {
            msg: "partial liquidation failure - reply (id 5)".to_string(),
        },
        err.downcast().unwrap()
    );

    // if quoteAssetAmountLimit == 273.8 < 68.455 * 4 = 273.82, quote asset gets is more than expected
    let msg = engine
        .liquidate(vamm.addr().to_string(), 1, Uint128::from(273_800_000u64))
        .unwrap();
    router.execute(carol.clone(), msg).unwrap();
}

#[test]
fn test_partially_liquidate_short_position() {
    let NativeTokenScenario {
        mut router,
        alice,
        bob,
        carol,
        owner,
        insurance_fund,
        engine,
        vamm,
        pricefeed,
        ..
    } = new_native_token_scenario();

    // set the latest price
    let price = Uint128::from(10_000_000u128);
    let timestamp = router.block_info().time.seconds();

    let msg = pricefeed
        .append_price("ETH".to_string(), price, timestamp)
        .unwrap();
    router.execute(owner.clone(), msg).unwrap();

    router.update_block(|block| {
        block.time = block.time.plus_seconds(900);
        block.height += 1;
    });

    let msg = engine
        .set_margin_ratios(Uint128::from(100_000u128))
        .unwrap();
    router.execute(owner.clone(), msg).unwrap();

    let msg = engine
        .set_partial_liquidation_ratio(Uint128::from(250_000u128))
        .unwrap();
    router.execute(owner.clone(), msg).unwrap();

    let msg = engine
        .set_liquidation_fee(Uint128::from(25_000u128))
        .unwrap();
    router.execute(owner.clone(), msg).unwrap();

    // when alice create a 20 margin * 10x position to get 25 short position
    // AMM after: 800 : 125
    let msg = engine
        .open_position(
            vamm.addr().to_string(),
            Side::Sell,
            Uint128::from(20_000_000u64),
            Uint128::from(10_000_000u64),
            Some(Uint128::from(7_000_000u64)),
            Some(Uint128::from(18_000_000u64)),
            Uint128::zero(),
            vec![Coin::new(20_000_000u128, "orai")],
        )
        .unwrap();
    router.execute(alice.clone(), msg).unwrap();

    router.update_block(|block| {
        block.time = block.time.plus_seconds(15);
        block.height += 1;
    });

    // when bob create a 19.67213115 margin * 1x position to get 3 long position
    // AMM after: 819.6721311 : 122
    let msg = engine
        .open_position(
            vamm.addr().to_string(),
            Side::Buy,
            Uint128::from(19_672_131u128),
            Uint128::from(1_000_000u64),
            Some(Uint128::from(10_000_000u64)),
            Some(Uint128::from(6_000_000u64)),
            Uint128::zero(),
            vec![Coin::new(19_672_131u128, "orai")],
        )
        .unwrap();
    router.execute(bob.clone(), msg).unwrap();

    let msg = engine
        .liquidate(vamm.addr().to_string(), 1, Uint128::zero())
        .unwrap();
    router.execute(carol.clone(), msg).unwrap();

    let position = engine
        .position(&router.wrap(), vamm.addr().to_string(), 1)
        .unwrap();
    assert_eq!(position.margin, Uint128::from(16_079_607u128));
    assert_eq!(position.size, Integer::new_negative(18_750_000u128));

    let margin_ratio = engine
        .get_margin_ratio(&router.wrap(), vamm.addr().to_string(), 1)
        .unwrap();
    assert_eq!(margin_ratio, Integer::new_positive(45_736u128));

    let carol_balance = router.wrap().query_balance(&carol, "orai").unwrap().amount;
    assert_eq!(carol_balance, Uint128::from(553_234u128));

    let insurance_balance = router
        .wrap()
        .query_balance(&insurance_fund.addr(), "orai")
        .unwrap()
        .amount;
    assert_eq!(insurance_balance, Uint128::from(5_000_553_234u128));
}

#[test]
fn test_partially_liquidate_short_position_with_quote_asset_limit() {
    let NativeTokenScenario {
        mut router,
        alice,
        bob,
        carol,
        owner,
        engine,
        vamm,
        pricefeed,
        ..
    } = new_native_token_scenario();

    // set the latest price
    let price = Uint128::from(10_000_000u128);
    let timestamp = router.block_info().time.seconds();

    let msg = pricefeed
        .append_price("ETH".to_string(), price, timestamp)
        .unwrap();
    router.execute(owner.clone(), msg).unwrap();

    router.update_block(|block| {
        block.time = block.time.plus_seconds(900);
        block.height += 1;
    });

    let msg = engine
        .set_margin_ratios(Uint128::from(100_000u128))
        .unwrap();
    router.execute(owner.clone(), msg).unwrap();

    let msg = engine
        .set_partial_liquidation_ratio(Uint128::from(250_000u128))
        .unwrap();
    router.execute(owner.clone(), msg).unwrap();

    let msg = engine
        .set_liquidation_fee(Uint128::from(25_000u128))
        .unwrap();
    router.execute(owner.clone(), msg).unwrap();

    // when alice create a 20 margin * 10x position to get 25 short position
    // AMM after: 800 : 125
    let msg = engine
        .open_position(
            vamm.addr().to_string(),
            Side::Sell,
            Uint128::from(20_000_000u64),
            Uint128::from(10_000_000u64),
            Some(Uint128::from(7_000_000u64)),
            Some(Uint128::from(18_000_000u64)),
            Uint128::zero(),
            vec![Coin::new(20_000_000u128, "orai")],
        )
        .unwrap();
    router.execute(alice.clone(), msg).unwrap();

    router.update_block(|block| {
        block.time = block.time.plus_seconds(15);
        block.height += 1;
    });

    // when bob create a 19.67213115 margin * 1x position to get 3 long position
    // AMM after: 819.6721311 : 122
    let msg = engine
        .open_position(
            vamm.addr().to_string(),
            Side::Buy,
            Uint128::from(19_672_131u128),
            Uint128::from(1_000_000u64),
            Some(Uint128::from(10_000_000u64)),
            Some(Uint128::from(5_000_000u64)),
            Uint128::zero(),
            vec![Coin::new(19_672_131u128, "orai")],
        )
        .unwrap();
    router.execute(bob.clone(), msg).unwrap();

    // partially liquidate 25%
    // liquidated positionNotional: getOutputPrice(25 (original position) * 0.25) = 44.258
    // if quoteAssetAmountLimit == 177 > 44.258 * 4 = 177.032, quote asset pays is more than expected, thus tx reverts
    let msg = engine
        .liquidate(vamm.addr().to_string(), 1, Uint128::from(177_000_000u64))
        .unwrap();
    let err = router.execute(carol.clone(), msg).unwrap_err();
    assert_eq!(
        StdError::GenericErr {
            msg: "partial liquidation failure - reply (id 5)".to_string(),
        },
        err.downcast().unwrap()
    );

    // if quoteAssetAmountLimit == 177.1 < 44.258 * 4 = 177.032, quote asset pays is less than expected
    let msg = engine
        .liquidate(vamm.addr().to_string(), 1, Uint128::from(177_100_000u64))
        .unwrap();
    router.execute(carol.clone(), msg).unwrap();
}

#[test]
fn test_long_position_complete_liquidation() {
    let NativeTokenScenario {
        mut router,
        alice,
        bob,
        carol,
        owner,
        insurance_fund,
        engine,
        vamm,
        pricefeed,
        ..
    } = new_native_token_scenario();

    // set the latest price
    let price = Uint128::from(10_000_000u128);
    let timestamp = router.block_info().time.seconds();

    let msg = pricefeed
        .append_price("ETH".to_string(), price, timestamp)
        .unwrap();
    router.execute(owner.clone(), msg).unwrap();

    router.update_block(|block| {
        block.time = block.time.plus_seconds(900);
        block.height += 1;
    });

    let msg = engine
        .set_margin_ratios(Uint128::from(100_000u128))
        .unwrap();
    router.execute(owner.clone(), msg).unwrap();

    let msg = engine
        .set_partial_liquidation_ratio(Uint128::from(250_000u128))
        .unwrap();
    router.execute(owner.clone(), msg).unwrap();

    let msg = engine
        .set_liquidation_fee(Uint128::from(25_000u128))
        .unwrap();
    router.execute(owner.clone(), msg).unwrap();

    // when alice create a 25 margin * 10x position to get 20 long position
    // AMM after: 1250 : 80
    let msg = engine
        .open_position(
            vamm.addr().to_string(),
            Side::Buy,
            Uint128::from(25_000_000u64),
            Uint128::from(10_000_000u64),
            Some(Uint128::from(14_000_000u64)),
            Some(Uint128::from(5_000_000u64)),
            Uint128::zero(),
            vec![Coin::new(25_000_000u128, "orai")],
        )
        .unwrap();
    router.execute(alice.clone(), msg).unwrap();

    router.update_block(|block| {
        block.time = block.time.plus_seconds(15);
        block.height += 1;
    });

    // when bob create a 73.52941176 margin * 1x position to get 3 short position
    // AMM after: 1176.470588 : 85
    let msg = engine
        .open_position(
            vamm.addr().to_string(),
            Side::Sell,
            Uint128::from(73_529_411u128),
            Uint128::from(1_000_000u64),
            Some(Uint128::from(6_000_000u64)),
            Some(Uint128::from(18_000_000u64)),
            Uint128::zero(),
            vec![Coin::new(73_529_411u128, "orai")],
        )
        .unwrap();
    router.execute(bob.clone(), msg).unwrap();

    let msg = engine
        .liquidate(vamm.addr().to_string(), 1, Uint128::zero())
        .unwrap();
    router.execute(carol.clone(), msg).unwrap();

    let err = engine
        .position(&router.wrap(), vamm.addr().to_string(), 1)
        .unwrap_err();
    assert_eq!(
        StdError::GenericErr {
            msg: "Querier contract error: margined_perp::margined_engine::Position not found"
                .to_string()
        },
        err
    );

    let carol_balance = router.wrap().query_balance(&carol, "orai").unwrap().amount;
    assert_eq!(carol_balance, Uint128::from(2_801_120u128));

    // 5000 - 0.91 - 2.8
    let insurance_balance = router
        .wrap()
        .query_balance(&insurance_fund.addr(), "orai")
        .unwrap()
        .amount;
    assert_eq!(insurance_balance, Uint128::from(4_996_288_516u128));
}

#[test]
fn test_long_position_complete_liquidation_with_slippage_limit() {
    let NativeTokenScenario {
        mut router,
        alice,
        bob,
        carol,
        owner,
        engine,
        vamm,
        pricefeed,
        ..
    } = new_native_token_scenario();

    // set the latest price
    let price = Uint128::from(10_000_000u128);
    let timestamp = router.block_info().time.seconds();

    let msg = pricefeed
        .append_price("ETH".to_string(), price, timestamp)
        .unwrap();
    router.execute(owner.clone(), msg).unwrap();

    router.update_block(|block| {
        block.time = block.time.plus_seconds(900);
        block.height += 1;
    });

    let msg = engine
        .set_margin_ratios(Uint128::from(100_000u128))
        .unwrap();
    router.execute(owner.clone(), msg).unwrap();

    let msg = engine
        .set_partial_liquidation_ratio(Uint128::from(250_000u128))
        .unwrap();
    router.execute(owner.clone(), msg).unwrap();

    let msg = engine
        .set_liquidation_fee(Uint128::from(25_000u128))
        .unwrap();
    router.execute(owner.clone(), msg).unwrap();

    // when alice create a 25 margin * 10x position to get 20 long position
    // AMM after: 1250 : 80
    let msg = engine
        .open_position(
            vamm.addr().to_string(),
            Side::Buy,
            Uint128::from(25_000_000u64),
            Uint128::from(10_000_000u64),
            Some(Uint128::from(14_000_000u64)),
            Some(Uint128::from(5_000_000u64)),
            Uint128::zero(),
            vec![Coin::new(25_000_000u128, "orai")],
        )
        .unwrap();
    router.execute(alice.clone(), msg).unwrap();

    router.update_block(|block| {
        block.time = block.time.plus_seconds(15);
        block.height += 1;
    });

    // when bob create a 73.52941176 margin * 1x position to get 3 short position
    // AMM after: 1176.470588 : 85
    let msg = engine
        .open_position(
            vamm.addr().to_string(),
            Side::Sell,
            Uint128::from(73_529_411u128),
            Uint128::from(1_000_000u64),
            Some(Uint128::from(6_000_000u64)),
            Some(Uint128::from(18_000_000u64)),
            Uint128::zero(),
            vec![Coin::new(73_529_411u128, "orai")],
        )
        .unwrap();
    router.execute(bob.clone(), msg).unwrap();

    let msg = engine
        .liquidate(vamm.addr().to_string(), 1, Uint128::from(224_100_000u128))
        .unwrap();
    let err = router.execute(carol.clone(), msg).unwrap_err();
    assert_eq!(
        StdError::GenericErr {
            msg: "liquidation failure - reply (id 4)".to_string(),
        },
        err.downcast().unwrap()
    );

    let msg = engine
        .liquidate(vamm.addr().to_string(), 1, Uint128::from(224_000_000u64))
        .unwrap();
    router.execute(carol.clone(), msg).unwrap();
}

#[test]
fn test_short_position_complete_liquidation() {
    let NativeTokenScenario {
        mut router,
        alice,
        bob,
        carol,
        owner,
        insurance_fund,
        engine,
        vamm,
        pricefeed,
        ..
    } = new_native_token_scenario();

    // set the latest price
    let price = Uint128::from(10_000_000u128);
    let timestamp = router.block_info().time.seconds();

    let msg = pricefeed
        .append_price("ETH".to_string(), price, timestamp)
        .unwrap();
    router.execute(owner.clone(), msg).unwrap();

    router.update_block(|block| {
        block.time = block.time.plus_seconds(900);
        block.height += 1;
    });

    let msg = engine
        .set_margin_ratios(Uint128::from(100_000u128))
        .unwrap();
    router.execute(owner.clone(), msg).unwrap();

    let msg = engine
        .set_partial_liquidation_ratio(Uint128::from(250_000u128))
        .unwrap();
    router.execute(owner.clone(), msg).unwrap();

    let msg = engine
        .set_liquidation_fee(Uint128::from(25_000u128))
        .unwrap();
    router.execute(owner.clone(), msg).unwrap();

    // when alice create a 20 margin * 10x position to get 25 short position
    // AMM after: 800 : 125
    let msg = engine
        .open_position(
            vamm.addr().to_string(),
            Side::Sell,
            Uint128::from(20_000_000u64),
            Uint128::from(10_000_000u64),
            Some(Uint128::from(6_000_000u64)),
            Some(Uint128::from(18_000_000u64)),
            Uint128::zero(),
            vec![Coin::new(20_000_000u128, "orai")],
        )
        .unwrap();
    router.execute(alice.clone(), msg).unwrap();

    router.update_block(|block| {
        block.time = block.time.plus_seconds(15);
        block.height += 1;
    });

    // when bob create a 40.33613445 margin * 1x position to get 3 long position
    // AMM after: 840.3361345 : 119
    let msg = engine
        .open_position(
            vamm.addr().to_string(),
            Side::Buy,
            Uint128::from(40_336_134u128),
            Uint128::from(1_000_000u64),
            Some(Uint128::from(14_000_000u64)),
            Some(Uint128::from(5_000_000u64)),
            Uint128::zero(),
            vec![Coin::new(40_336_134u128, "orai")],
        )
        .unwrap();
    router.execute(bob.clone(), msg).unwrap();

    let msg = engine
        .liquidate(vamm.addr().to_string(), 1, Uint128::zero())
        .unwrap();
    router.execute(carol.clone(), msg).unwrap();

    let err = engine
        .position(&router.wrap(), vamm.addr().to_string(), 1)
        .unwrap_err();
    assert_eq!(
        StdError::GenericErr {
            msg: "Querier contract error: margined_perp::margined_engine::Position not found"
                .to_string()
        },
        err
    );
    let carol_balance = router.wrap().query_balance(&carol, "orai").unwrap().amount;
    assert_eq!(carol_balance, Uint128::from(2_793_670u128));

    // 5000 - 3.49 - 2.79
    let insurance_balance = router
        .wrap()
        .query_balance(&insurance_fund.addr(), "orai")
        .unwrap()
        .amount;
    assert_eq!(insurance_balance, Uint128::from(4_993_712_679u128));
}

#[test]
fn test_force_error_position_not_liquidation_twap_over_maintenance_margin() {
    let NativeTokenScenario {
        mut router,
        alice,
        bob,
        carol,
        owner,
        engine,
        vamm,
        pricefeed,
        ..
    } = new_native_token_scenario();

    // set the latest price
    let price = Uint128::from(10_000_000u128);
    let timestamp = router.block_info().time.seconds();

    let msg = pricefeed
        .append_price("ETH".to_string(), price, timestamp)
        .unwrap();
    router.execute(owner.clone(), msg).unwrap();

    router.update_block(|block| {
        block.time = block.time.plus_seconds(900);
        block.height += 1;
    });

    let msg = engine
        .set_margin_ratios(Uint128::from(100_000u128))
        .unwrap();
    router.execute(owner.clone(), msg).unwrap();

    let msg = engine
        .set_partial_liquidation_ratio(Uint128::from(250_000u128))
        .unwrap();
    router.execute(owner.clone(), msg).unwrap();

    let msg = engine
        .set_liquidation_fee(Uint128::from(25_000u128))
        .unwrap();
    router.execute(owner.clone(), msg).unwrap();

    // when bob create a 20 margin * 5x long position when 9.0909090909 quoteAsset = 100
    // AMM after: 1100 : 90.9090909091
    let msg = engine
        .open_position(
            vamm.addr().to_string(),
            Side::Buy,
            Uint128::from(20_000_000u64),
            Uint128::from(5_000_000u64),
            Some(Uint128::from(14_000_000u64)),
            Some(Uint128::from(9_000_000u64)),
            Uint128::zero(),
            vec![Coin::new(20_000_000u128, "orai")],
        )
        .unwrap();
    router.execute(bob.clone(), msg).unwrap();

    router.update_block(|block| {
        block.time = block.time.plus_seconds(15);
        block.height += 1;
    });

    // when alice create a 20 margin * 5x long position when 7.5757575758 quoteAsset = 100
    // AMM after: 1200 : 83.3333333333
    let msg = engine
        .open_position(
            vamm.addr().to_string(),
            Side::Buy,
            Uint128::from(20_000_000u64),
            Uint128::from(5_000_000u64),
            Some(Uint128::from(16_000_000u64)),
            Some(Uint128::from(10_000_000u64)),
            Uint128::zero(),
            vec![Coin::new(20_000_000u128, "orai")],
        )
        .unwrap();
    router.execute(alice.clone(), msg).unwrap();

    router.update_block(|block| {
        block.time = block.time.plus_seconds(600);
        block.height += 1;
    });

    // when bob sell his position when 7.5757575758 quoteAsset = 100
    // AMM after: 1100 : 90.9090909091
    let msg = engine
        .open_position(
            vamm.addr().to_string(),
            Side::Sell,
            Uint128::from(20_000_000u64),
            Uint128::from(5_000_000u64),
            Some(Uint128::from(6_000_000u64)),
            Some(Uint128::from(18_000_000u64)),
            Uint128::zero(),
            vec![Coin::new(20_000_000u128, "orai")],
        )
        .unwrap();
    router.execute(bob.clone(), msg).unwrap();

    router.update_block(|block| {
        block.time = block.time.plus_seconds(15);
        block.height += 1;
    });

    // verify alice's openNotional = 100
    // spot price PnL = positionValue - openNotional = 84.62 - 100 = -15.38
    // TWAP PnL = (70.42 * 270 + 84.62 * 15 + 99.96 * 600 + 84.62 * 15) / 900 - 100 ~= -9.39
    // Use TWAP price PnL since -9.39 > -15.38
    let position = engine
        .position(&router.wrap(), vamm.addr().to_string(), 2)
        .unwrap();
    assert_eq!(position.notional, Uint128::from(100_000_000u64));

    let pnl = engine
        .get_unrealized_pnl(
            &router.wrap(),
            vamm.addr().to_string(),
            2,
            PnlCalcOption::SpotPrice,
        )
        .unwrap();
    assert_eq!(pnl.unrealized_pnl, Integer::new_negative(15_384_623u128));

    let pnl = engine
        .get_unrealized_pnl(
            &router.wrap(),
            vamm.addr().to_string(),
            2,
            PnlCalcOption::Twap,
        )
        .unwrap();
    assert_eq!(pnl.unrealized_pnl, Integer::new_negative(9_386_068u128));

    let price = vamm.spot_price(&router.wrap()).unwrap();
    let msg = pricefeed
        .append_price("ETH".to_string(), price, timestamp)
        .unwrap();
    router.execute(owner.clone(), msg).unwrap();

    let msg = engine
        .liquidate(vamm.addr().to_string(), 2, Uint128::zero())
        .unwrap();
    let err = router.execute(carol.clone(), msg).unwrap_err();
    assert_eq!(
        StdError::GenericErr {
            msg: "Position is overcollateralized".to_string(),
        },
        err.downcast().unwrap()
    );
}

#[test]
fn test_force_error_position_not_liquidation_spot_over_maintenance_margin() {
    let NativeTokenScenario {
        mut router,
        alice,
        carol,
        owner,
        engine,
        vamm,
        pricefeed,
        ..
    } = new_native_token_scenario();

    // set the latest price
    let price = Uint128::from(10_000_000u128);
    let timestamp = router.block_info().time.seconds();

    let msg = pricefeed
        .append_price("ETH".to_string(), price, timestamp)
        .unwrap();
    router.execute(owner.clone(), msg).unwrap();

    router.update_block(|block| {
        block.time = block.time.plus_seconds(900);
        block.height += 1;
    });

    let msg = engine
        .set_margin_ratios(Uint128::from(100_000u128))
        .unwrap();
    router.execute(owner.clone(), msg).unwrap();

    let msg = engine
        .set_partial_liquidation_ratio(Uint128::from(250_000u128))
        .unwrap();
    router.execute(owner.clone(), msg).unwrap();

    let msg = engine
        .set_liquidation_fee(Uint128::from(25_000u128))
        .unwrap();
    router.execute(owner.clone(), msg).unwrap();

    // when alice create a 20 margin * 5x long position when 9.0909090909 quoteAsset = 100
    // AMM after: 1100 : 90.9090909091
    let msg = engine
        .open_position(
            vamm.addr().to_string(),
            Side::Buy,
            Uint128::from(20_000_000u64),
            Uint128::from(5_000_000u64),
            Some(to_decimals(10)),
            Some(Uint128::zero()),
            Uint128::zero(),
            vec![Coin::new(20_000_000u128, "orai")],
        )
        .unwrap();
    router.execute(alice.clone(), msg).unwrap();

    router.update_block(|block| {
        block.time = block.time.plus_seconds(15);
        block.height += 1;
    });

    // verify alice's openNotional = 100
    // spot price PnL = positionValue - openNotional = 100 - 100 = 0
    // TWAP PnL = (83.3333333333 * 885 + 100 * 15) / 900 - 100 = -16.39
    // Use spot price PnL since 0 > -16.39
    let position = engine
        .position(&router.wrap(), vamm.addr().to_string(), 1)
        .unwrap();
    assert_eq!(position.notional, Uint128::from(100_000_000u64));

    // workaround: rounding error, should be 0 but it's actually 10 wei
    let pnl = engine
        .get_unrealized_pnl(
            &router.wrap(),
            vamm.addr().to_string(),
            1,
            PnlCalcOption::SpotPrice,
        )
        .unwrap();
    assert_eq!(pnl.unrealized_pnl, Integer::new_negative(1u128));

    let pnl = engine
        .get_unrealized_pnl(
            &router.wrap(),
            vamm.addr().to_string(),
            1,
            PnlCalcOption::Twap,
        )
        .unwrap();
    assert_eq!(pnl.unrealized_pnl, Integer::new_negative(16_388_891u128));

    let price = vamm.spot_price(&router.wrap()).unwrap();
    let msg = pricefeed
        .append_price("ETH".to_string(), price, timestamp)
        .unwrap();
    router.execute(owner.clone(), msg).unwrap();

    let msg = engine
        .liquidate(vamm.addr().to_string(), 1, Uint128::zero())
        .unwrap();
    let err = router.execute(carol.clone(), msg).unwrap_err();
    assert_eq!(
        StdError::GenericErr {
            msg: "Position is overcollateralized".to_string(),
        },
        err.downcast().unwrap()
    );
}

#[test]
fn test_force_error_empty_position() {
    let NativeTokenScenario {
        mut router,
        carol,
        owner,
        engine,
        vamm,
        ..
    } = new_native_token_scenario();

    let msg = engine
        .set_margin_ratios(Uint128::from(100_000u128))
        .unwrap();
    router.execute(owner.clone(), msg).unwrap();

    let msg = engine
        .set_partial_liquidation_ratio(Uint128::from(250_000u128))
        .unwrap();
    router.execute(owner.clone(), msg).unwrap();

    let msg = engine
        .set_liquidation_fee(Uint128::from(25_000u128))
        .unwrap();
    router.execute(owner.clone(), msg).unwrap();

    let msg = engine
        .liquidate(vamm.addr().to_string(), 1, Uint128::zero())
        .unwrap();
    let err = router.execute(carol.clone(), msg).unwrap_err();

    assert_eq!(
        err.source().unwrap().to_string(),
        "margined_perp::margined_engine::Position not found"
    );
}

#[test]
fn test_partially_liquidate_one_position_within_fluctuation_limit() {
    let mut env = new_native_token_scenario();

    // set the latest price
    let price = Uint128::from(10_000_000u128);
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
        .set_margin_ratios(Uint128::from(100_000u128))
        .unwrap();
    env.router.execute(env.owner.clone(), msg).unwrap();

    let msg = env
        .engine
        .set_partial_liquidation_ratio(Uint128::from(250_000u128))
        .unwrap();
    env.router.execute(env.owner.clone(), msg).unwrap();

    let msg = env
        .engine
        .set_liquidation_fee(Uint128::from(25_000u128))
        .unwrap();
    env.router.execute(env.owner.clone(), msg).unwrap();

    // when bob create a 20 margin * 5x long position when 9.0909090909 quoteAsset = 100
    // AMM after: 1100 : 90.9090909091
    env.open_small_position(
        env.bob.clone(),
        Side::Buy,
        Uint128::from(20_000_000u64),
        Uint128::from(5_000_000u64),
        Some(Uint128::from(16_000_000u64)),
        Some(Uint128::from(8_000_000u64)),
        1u64,
    );

    // when alice create a 20 margin * 5x long position when 7.5757575758 quoteAsset = 100
    // AMM after: 1200 : 83.3333333333
    // alice get: 90.9090909091 - 83.3333333333 = 7.5757575758
    env.open_small_position(
        env.alice.clone(),
        Side::Buy,
        Uint128::from(20_000_000u64),
        Uint128::from(5_000_000u64),
        Some(Uint128::from(14_000_000u64)),
        Some(Uint128::from(10_000_000u64)),
        1u64,
    );

    // AMM after: 1100 : 90.9090909091, price: 12.1
    env.open_small_position(
        env.bob.clone(),
        Side::Sell,
        Uint128::from(20_000_000u64),
        Uint128::from(5_000_000u64),
        Some(Uint128::from(11_000_000u64)),
        Some(Uint128::from(20_000_000u64)),
        1u64,
    );

    let price = env.vamm.spot_price(&env.router.wrap()).unwrap();
    let msg = env
        .pricefeed
        .append_price("ETH".to_string(), price, timestamp)
        .unwrap();
    env.router.execute(env.owner.clone(), msg).unwrap();

    // liquidate -> return 25% base asset to AMM
    // 90.9090909091 + 1.89 = 92.8
    // AMM after: 1077.55102 : 92.8, price: 11.61
    // fluctuation: (12.1 - 11.61116202) / 12.1 = 0.04039983306
    // values can be retrieved with amm.quoteAssetReserve() & amm.baseAssetReserve()
    let msg = env
        .engine
        .liquidate(env.vamm.addr().to_string(), 2, Uint128::zero())
        .unwrap();
    env.router.execute(env.carol.clone(), msg).unwrap();

    let state = env.vamm.state(&env.router.wrap()).unwrap();
    assert_eq!(state.quote_asset_reserve, Uint128::from(1_077_551_026u128));
    assert_eq!(state.base_asset_reserve, Uint128::from(92_803_031u128));
}

#[test]
fn test_partially_liquidate_two_positions_within_fluctuation_limit() {
    let mut env = new_native_token_scenario();

    // set the latest price
    let price = Uint128::from(10_000_000u128);
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

    let msg = CosmosMsg::Bank(BankMsg::Send {
        to_address: env.carol.to_string(),
        amount: vec![Coin::new(1_000u128 * 10u128.pow(6), "orai")],
    });
    env.router.execute(env.bank.clone(), msg).unwrap();

    let msg = env
        .vamm
        .set_fluctuation_limit_ratio(Uint128::from(41_000u128))
        .unwrap();
    env.router.execute(env.owner.clone(), msg).unwrap();

    let msg = env
        .engine
        .set_margin_ratios(Uint128::from(199_996u128))
        .unwrap();
    env.router.execute(env.owner.clone(), msg).unwrap();

    let msg = env
        .engine
        .set_partial_liquidation_ratio(Uint128::from(250_000u128))
        .unwrap();
    env.router.execute(env.owner.clone(), msg).unwrap();

    let msg = env
        .engine
        .set_liquidation_fee(Uint128::from(25_000u128))
        .unwrap();
    env.router.execute(env.owner.clone(), msg).unwrap();

    // when bob create a 20 margin * 5x long position when 9.0909090909 quoteAsset = 100
    // AMM after: 1100 : 90.9090909091
    // actual margin ratio is 19.99...9%
    env.open_small_position(
        env.bob.clone(),
        Side::Buy,
        Uint128::from(4_000_000u64),
        Uint128::from(5_000_000u64),
        Some(Uint128::from(15_000_000u64)),
        Some(Uint128::from(8_000_000u64)),
        5u64,
    );

    // when carol create a 10 margin * 5x long position when 7.5757575758 quoteAsset = 100
    // AMM after: quote = 1150
    env.open_small_position(
        env.carol.clone(),
        Side::Buy,
        Uint128::from(2_000_000u64),
        Uint128::from(5_000_000u64),
        Some(Uint128::from(19_000_000u64)),
        Some(Uint128::from(11_000_000u64)),
        5u64,
    );

    // when alice create a 10 margin * 5x long position
    // AMM after: quote = 1200
    env.open_small_position(
        env.alice.clone(),
        Side::Buy,
        Uint128::from(2_000_000u64),
        Uint128::from(5_000_000u64),
        Some(Uint128::from(23_000_000u64)),
        Some(Uint128::from(13_000_000u64)),
        5u64,
    );

    // bob short 100
    // AMM after: 1100 : 90.9090909091, price: 12.1
    env.open_small_position(
        env.bob.clone(),
        Side::Sell,
        Uint128::from(4_000_000u64),
        Uint128::from(5_000_000u64),
        Some(Uint128::from(10_000_000u64)),
        Some(Uint128::from(15_000_000u64)),
        5u64,
    );

    let price = env.vamm.spot_price(&env.router.wrap()).unwrap();
    let msg = env
        .pricefeed
        .append_price("ETH".to_string(), price, timestamp)
        .unwrap();
    env.router.execute(env.owner.clone(), msg).unwrap();

    // AMM after: 1077.55102 : 92.8, price: 11.61
    // fluctuation: (12.1 - 11.61116202) / 12.1 = 0.04039983306
    let msg = env
        .engine
        .liquidate(env.vamm.addr().to_string(), 11, Uint128::zero())
        .unwrap();
    env.router.execute(env.carol.clone(), msg).unwrap();

    let msg = env
        .engine
        .liquidate(env.vamm.addr().to_string(), 12, Uint128::zero())
        .unwrap();
    env.router.execute(env.carol.clone(), msg).unwrap();

    let msg = env
        .engine
        .liquidate(env.vamm.addr().to_string(), 13, Uint128::zero())
        .unwrap();
    env.router.execute(env.carol.clone(), msg).unwrap();

    let msg = env
        .engine
        .liquidate(env.vamm.addr().to_string(), 14, Uint128::zero())
        .unwrap();
    env.router.execute(env.carol.clone(), msg).unwrap();

    let msg = env
        .engine
        .liquidate(env.vamm.addr().to_string(), 15, Uint128::zero())
        .unwrap();
    env.router.execute(env.carol.clone(), msg).unwrap();

    let msg = env
        .engine
        .liquidate(env.vamm.addr().to_string(), 6, Uint128::zero())
        .unwrap();
    env.router.execute(env.bob.clone(), msg).unwrap();

    let msg = env
        .engine
        .liquidate(env.vamm.addr().to_string(), 7, Uint128::zero())
        .unwrap();
    env.router.execute(env.bob.clone(), msg).unwrap();

    let msg = env
        .engine
        .liquidate(env.vamm.addr().to_string(), 8, Uint128::zero())
        .unwrap();
    env.router.execute(env.bob.clone(), msg).unwrap();

    let msg = env
        .engine
        .liquidate(env.vamm.addr().to_string(), 9, Uint128::zero())
        .unwrap();
    env.router.execute(env.bob.clone(), msg).unwrap();

    let msg = env
        .engine
        .liquidate(env.vamm.addr().to_string(), 10, Uint128::zero())
        .unwrap();
    env.router.execute(env.bob.clone(), msg).unwrap();

    let state = env.vamm.state(&env.router.wrap()).unwrap();
    assert_eq!(state.quote_asset_reserve, Uint128::from(1_077_551_079u128));
    assert_eq!(state.base_asset_reserve, Uint128::from(92_803_034u128));
}

#[test]
fn test_partially_liquidate_three_positions_within_fluctuation_limit() {
    let mut env = new_native_token_scenario();

    // set the latest price
    let price = Uint128::from(10_000_000u128);
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
        .vamm
        .set_fluctuation_limit_ratio(Uint128::from(210_000u128))
        .unwrap();
    env.router.execute(env.owner.clone(), msg).unwrap();

    let msg = env
        .engine
        .set_margin_ratios(Uint128::from(199_995u128))
        .unwrap();
    env.router.execute(env.owner.clone(), msg).unwrap();

    let msg = env
        .engine
        .set_partial_liquidation_ratio(Uint128::from(250_000u128))
        .unwrap();
    env.router.execute(env.owner.clone(), msg).unwrap();

    let msg = env
        .engine
        .set_liquidation_fee(Uint128::from(25_000u128))
        .unwrap();
    env.router.execute(env.owner.clone(), msg).unwrap();

    // mint funds for carol
    let msg = CosmosMsg::Bank(BankMsg::Send {
        to_address: env.carol.to_string(),
        amount: vec![Coin::new(1_000u128 * 10u128.pow(6), "orai")],
    });
    env.router.execute(env.bank.clone(), msg).unwrap();

    // when bob create a 20 margin * 5x long position when 9.0909090909 quoteAsset = 100
    // AMM after: 1100 : 90.9090909091
    // actual margin ratio is 19.99...9%
    env.open_small_position(
        env.bob.clone(),
        Side::Buy,
        Uint128::from(20_000_000u64),
        Uint128::from(5_000_000u64),
        Some(Uint128::from(16_000_000u64)),
        Some(Uint128::from(8_000_000u64)),
        1u64,
    );

    // when carol create a 10 margin * 5x long position when 7.5757575758 quoteAsset = 100
    // AMM after: quote = 1150
    env.open_small_position(
        env.carol.clone(),
        Side::Buy,
        Uint128::from(10_000_000u64),
        Uint128::from(5_000_000u64),
        Some(Uint128::from(14_000_000u64)),
        Some(Uint128::from(11_000_000u64)),
        1u64,
    );

    // when alice create a 10 margin * 5x long position
    // AMM after: quote = 1200
    env.open_small_position(
        env.alice.clone(),
        Side::Buy,
        Uint128::from(10_000_000u64),
        Uint128::from(5_000_000u64),
        Some(Uint128::from(17_000_000u64)),
        Some(Uint128::from(13_000_000u64)),
        1u64,
    );

    // when david create a 2 margin * 5x long position
    // AMM after: quote = 1210 : 82.6446281
    // alice + carol + david get: 90.9090909091 - 82.6446281 = 8.2644628091
    env.open_small_position(
        env.david.clone(),
        Side::Buy,
        Uint128::from(2_000_000u128), // 0.4
        Uint128::from(5_000_000u64),
        Some(Uint128::from(20_000_000u64)),
        Some(Uint128::from(13_000_000u64)),
        1u64,
    );

    // AMM after: 1110 : 90.09009009, price: 12.321
    env.open_small_position(
        env.bob.clone(),
        Side::Sell,
        Uint128::from(20_000_000u64),
        Uint128::from(5_000_000u64),
        Some(Uint128::from(13_000_000u64)),
        Some(Uint128::from(20_000_000u64)),
        1u64,
    );

    let bob_balance = env
        .router
        .wrap()
        .query_balance(&env.bob, "orai")
        .unwrap()
        .amount;
    assert_eq!(bob_balance, Uint128::from(4_960_000_000u128));

    let price = env.vamm.spot_price(&env.router.wrap()).unwrap();
    let msg = env
        .pricefeed
        .append_price("ETH".to_string(), price, timestamp)
        .unwrap();
    env.router.execute(env.owner.clone(), msg).unwrap();

    // AMM after: close to 1079.066031 : 92.67273, price: 11.64383498
    // fluctuation: (12.321 - 11.64383498) / 12.321 = 0.05496023212
    let msg = env
        .engine
        .liquidate(env.vamm.addr().to_string(), 3, Uint128::zero())
        .unwrap();
    env.router.execute(env.bob.clone(), msg).unwrap();

    let msg = env
        .engine
        .liquidate(env.vamm.addr().to_string(), 2, Uint128::zero())
        .unwrap();
    env.router.execute(env.bob.clone(), msg).unwrap();

    let msg = env
        .engine
        .liquidate(env.vamm.addr().to_string(), 4, Uint128::zero())
        .unwrap();
    env.router.execute(env.bob.clone(), msg).unwrap();

    let state = env.vamm.state(&env.router.wrap()).unwrap();
    assert_eq!(state.quote_asset_reserve, Uint128::from(1_079_066_040u128));
    assert_eq!(state.base_asset_reserve, Uint128::from(92_672_736u128));
}

#[test]
fn test_partially_liquidate_two_positions_and_completely_liquidate_one_within_fluctuation_limit() {
    let mut env = new_native_token_scenario();

    // set the latest price
    let price = Uint128::from(10_000_000u128);
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
        .vamm
        .set_fluctuation_limit_ratio(Uint128::from(210_000u128))
        .unwrap();
    env.router.execute(env.owner.clone(), msg).unwrap();

    let msg = env
        .engine
        .set_margin_ratios(Uint128::from(199_999u128))
        .unwrap();
    env.router.execute(env.owner.clone(), msg).unwrap();

    let msg = env
        .engine
        .set_partial_liquidation_ratio(Uint128::from(250_000u128))
        .unwrap();
    env.router.execute(env.owner.clone(), msg).unwrap();

    let msg = env
        .engine
        .set_liquidation_fee(Uint128::from(25_000u128))
        .unwrap();
    env.router.execute(env.owner.clone(), msg).unwrap();

    // mint funds for carol
    let msg = CosmosMsg::Bank(BankMsg::Send {
        to_address: env.carol.to_string(),
        amount: vec![Coin::new(1_000u128 * 10u128.pow(6), "orai")],
    });
    env.router.execute(env.bank.clone(), msg).unwrap();

    // when bob create a 20 margin * 5x long position when 9.0909090909 quoteAsset = 100
    // AMM after: 1100 : 90.9090909091
    env.open_small_position(
        env.bob.clone(),
        Side::Buy,
        Uint128::from(20_000_000u64),
        Uint128::from(5_000_000u64),
        Some(Uint128::from(16_000_000u64)),
        Some(Uint128::from(8_000_000u64)),
        1u64,
    );

    // when carol create a 10 margin * 5x long position when 7.5757575758 quoteAsset = 100
    // AMM after: quote = 1150 : 86.9565217391
    env.open_small_position(
        env.carol.clone(),
        Side::Buy,
        Uint128::from(10_000_000u64),
        Uint128::from(5_000_000u64),
        Some(Uint128::from(13_000_000u64)),
        Some(Uint128::from(11_000_000u64)),
        1u64,
    );

    // when alice create a 10 margin * 5x long position
    // AMM after: quote = 1200
    env.open_small_position(
        env.alice.clone(),
        Side::Buy,
        Uint128::from(10_000_000u64),
        Uint128::from(5_000_000u64),
        Some(Uint128::from(14_000_000u64)),
        Some(Uint128::from(12_000_000u64)),
        1u64,
    );

    // when david create a 10 margin * 5x long position
    // AMM after: quote = 1250 : 80
    // alice + carol + david get: 90.9090909091 - 80 = 10.9090909091
    env.open_small_position(
        env.david.clone(),
        Side::Buy,
        Uint128::from(10_000_000u64),
        Uint128::from(5_000_000u64),
        Some(Uint128::from(18_000_000u64)),
        Some(Uint128::from(13_000_000u64)),
        1u64,
    );

    // AMM after: 1150 : 86.9565217391, price: 13.225
    env.open_small_position(
        env.bob.clone(),
        Side::Sell,
        Uint128::from(20_000_000u64),
        Uint128::from(5_000_000u64),
        Some(Uint128::from(8_000_000u64)),
        Some(Uint128::from(20_000_000u64)),
        1u64,
    );

    let price = env.vamm.spot_price(&env.router.wrap()).unwrap();
    let msg = env
        .pricefeed
        .append_price("ETH".to_string(), price, timestamp)
        .unwrap();
    env.router.execute(env.owner.clone(), msg).unwrap();

    // alice's & carol's positions are partially closed, while relayer's position is closed completely
    // AMM after: close to 1084.789366 : 92.1837, price: 11.7676797
    // fluctuation: (13.225 - 11.7676797) / 13.225 = 0.1101943516
    let msg = env
        .engine
        .liquidate(env.vamm.addr().to_string(), 3, Uint128::zero())
        .unwrap();
    env.router.execute(env.bob.clone(), msg).unwrap();

    let msg = env
        .engine
        .liquidate(env.vamm.addr().to_string(), 2, Uint128::zero())
        .unwrap();
    env.router.execute(env.bob.clone(), msg).unwrap();

    let msg = env
        .engine
        .liquidate(env.vamm.addr().to_string(), 4, Uint128::zero())
        .unwrap();
    env.router.execute(env.bob.clone(), msg).unwrap();

    let state = env.vamm.state(&env.router.wrap()).unwrap();
    assert_eq!(state.quote_asset_reserve, Uint128::from(1_084_789_377u128));
    assert_eq!(state.base_asset_reserve, Uint128::from(92_183_795u128));
}

#[test]
fn test_liquidate_one_position_exceeding_fluctuation_limit() {
    let mut env = new_native_token_scenario();

    // set the latest price
    let price = Uint128::from(10_000_000u128);
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
        .vamm
        .set_fluctuation_limit_ratio(Uint128::from(210_000u128))
        .unwrap();
    env.router.execute(env.owner.clone(), msg).unwrap();

    let msg = env
        .engine
        .set_margin_ratios(Uint128::from(100_000u128))
        .unwrap();
    env.router.execute(env.owner.clone(), msg).unwrap();

    let msg = env
        .engine
        .set_partial_liquidation_ratio(Uint128::from(250_000u128))
        .unwrap();
    env.router.execute(env.owner.clone(), msg).unwrap();

    let msg = env
        .engine
        .set_liquidation_fee(Uint128::from(25_000u128))
        .unwrap();
    env.router.execute(env.owner.clone(), msg).unwrap();

    // when bob create a 20 margin * 5x long position when 9.0909090909 quoteAsset = 100
    // AMM after: 1100 : 90.9090909091
    env.open_small_position(
        env.bob.clone(),
        Side::Buy,
        Uint128::from(20_000_000u64),
        Uint128::from(5_000_000u64),
        Some(Uint128::from(16_000_000u64)),
        Some(Uint128::from(8_000_000u64)),
        1u64,
    );

    // when alice create a 20 margin * 5x long position when 7.5757575758 quoteAsset = 100
    // AMM after: 1200 : 83.3333333333
    // alice get: 90.9090909091 - 83.3333333333 = 7.5757575758
    env.open_small_position(
        env.alice.clone(),
        Side::Buy,
        Uint128::from(20_000_000u64),
        Uint128::from(5_000_000u64),
        Some(Uint128::from(15_000_000u64)),
        Some(Uint128::from(11_000_000u64)),
        1u64,
    );

    // AMM after: 1100 : 90.9090909091, price: 12.1
    env.open_small_position(
        env.bob.clone(),
        Side::Sell,
        Uint128::from(20_000_000u64),
        Uint128::from(5_000_000u64),
        Some(Uint128::from(10_000_000u64)),
        Some(Uint128::from(15_000_000u64)),
        1u64,
    );

    let price = env.vamm.spot_price(&env.router.wrap()).unwrap();
    let msg = env
        .pricefeed
        .append_price("ETH".to_string(), price, timestamp)
        .unwrap();
    env.router.execute(env.owner.clone(), msg).unwrap();

    // liquidate -> return 25% base asset to AMM
    // 90.9090909091 + 1.89 = 92.8
    // AMM after: 1077.55102 : 92.8, price: 11.61
    // fluctuation: (12.1 - 11.61116202) / 12.1 = 0.04039983306
    // values can be retrieved with amm.quoteAssetReserve() & amm.baseAssetReserve()
    let msg = env
        .engine
        .liquidate(env.vamm.addr().to_string(), 2, Uint128::zero())
        .unwrap();
    let response = env.router.execute(env.carol.clone(), msg).unwrap();
    assert_eq!(
        response.events[5].attributes[1].value,
        "partial_liquidation_reply".to_string()
    );
}

#[test]
fn test_partially_liquidate_one_position_exceeding_fluctuation_limit() {
    let mut env = new_native_token_scenario();

    // set the latest price
    let price = Uint128::from(10_000_000u128);
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
        .vamm
        .set_fluctuation_limit_ratio(Uint128::from(500_000u128))
        .unwrap();
    env.router.execute(env.owner.clone(), msg).unwrap();

    let msg = env
        .engine
        .set_margin_ratios(Uint128::from(100_000u128))
        .unwrap();
    env.router.execute(env.owner.clone(), msg).unwrap();

    let msg = env
        .engine
        .set_partial_liquidation_ratio(Uint128::from(250_000u128))
        .unwrap();
    env.router.execute(env.owner.clone(), msg).unwrap();

    let msg = env
        .engine
        .set_liquidation_fee(Uint128::from(25_000u128))
        .unwrap();
    env.router.execute(env.owner.clone(), msg).unwrap();

    // when bob create a 20 margin * 5x long position when 9.0909090909 quoteAsset = 100
    // AMM after: 1100 : 90.9090909091
    env.open_small_position(
        env.bob.clone(),
        Side::Buy,
        Uint128::from(20_000_000u64),
        Uint128::from(5_000_000u64),
        Some(Uint128::from(16_000_000u64)),
        Some(Uint128::from(8_000_000u64)),
        1u64,
    );

    // when alice create a 20 margin * 5x long position when 7.5757575758 quoteAsset = 100
    // AMM after: 1200 : 83.3333333333
    // alice get: 90.9090909091 - 83.3333333333 = 7.5757575758
    env.open_small_position(
        env.alice.clone(),
        Side::Buy,
        Uint128::from(20_000_000u64),
        Uint128::from(5_000_000u64),
        Some(Uint128::from(15_000_000u64)),
        Some(Uint128::from(11_000_000u64)),
        1u64,
    );

    // AMM after: 1100 : 90.9090909091, price: 12.1
    env.open_small_position(
        env.bob.clone(),
        Side::Sell,
        Uint128::from(20_000_000u64),
        Uint128::from(5_000_000u64),
        Some(Uint128::from(10_000_000u64)),
        Some(Uint128::from(15_000_000u64)),
        1u64,
    );

    let msg = env
        .vamm
        .set_fluctuation_limit_ratio(Uint128::from(70_000u128))
        .unwrap();
    env.router.execute(env.owner.clone(), msg).unwrap();

    let msg = env
        .engine
        .open_position(
            env.vamm.addr().to_string(),
            Side::Sell,
            Uint128::from(44_000_000u64),
            Uint128::from(1_000_000u64),
            Some(Uint128::from(9_000_000u64)),
            Some(Uint128::from(13_000_000u64)),
            Uint128::zero(),
            vec![Coin::new(25_000_000u128, "orai")],
        )
        .unwrap();
    let err = env.router.execute(env.alice.clone(), msg).unwrap_err();
    assert_eq!(
        StdError::GenericErr {
            msg: format!(
                "open position failure - reply (id {})",
                INCREASE_POSITION_REPLY_ID
            )
        },
        err.downcast().unwrap()
    );

    let msg = env
        .engine
        .set_margin_ratios(Uint128::from(100_000u128))
        .unwrap();
    env.router.execute(env.owner.clone(), msg).unwrap();

    let price = env.vamm.spot_price(&env.router.wrap()).unwrap();
    let msg = env
        .pricefeed
        .append_price("ETH".to_string(), price, timestamp)
        .unwrap();
    env.router.execute(env.owner.clone(), msg).unwrap();

    // liquidate -> return 25% base asset to AMM
    // 90.9090909091 + 1.89 = 92.8
    // AMM after: 1077.55102 : 92.8, price: 11.61
    // fluctuation: (12.1 - 11.61116202) / 12.1 = 0.04039983306
    // values can be retrieved with amm.quoteAssetReserve() & amm.baseAssetReserve()
    let msg = env
        .engine
        .liquidate(env.vamm.addr().to_string(), 2, Uint128::zero())
        .unwrap();
    let response = env.router.execute(env.carol.clone(), msg).unwrap();
    assert_eq!(
        response.events[5].attributes[1].value,
        "partial_liquidation_reply".to_string()
    );
}

#[test]
fn test_force_error_partially_liquidate_two_positions_exceeding_fluctuation_limit() {
    let mut env = new_native_token_scenario();

    // set the latest price
    let price = Uint128::from(10_000_000u128);
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
        .vamm
        .set_fluctuation_limit_ratio(Uint128::from(147_000u128))
        .unwrap();
    env.router.execute(env.owner.clone(), msg).unwrap();

    let msg = env
        .engine
        .set_margin_ratios(Uint128::from(199_999u128))
        .unwrap();
    env.router.execute(env.owner.clone(), msg).unwrap();

    let msg = env
        .engine
        .set_partial_liquidation_ratio(Uint128::from(500_000u128))
        .unwrap();
    env.router.execute(env.owner.clone(), msg).unwrap();

    let msg = env
        .engine
        .set_liquidation_fee(Uint128::from(25_000u128))
        .unwrap();
    env.router.execute(env.owner.clone(), msg).unwrap();

    // mint funds for carol
    let msg = CosmosMsg::Bank(BankMsg::Send {
        to_address: env.carol.to_string(),
        amount: vec![Coin::new(1_000u128 * 10u128.pow(6), "orai")],
    });
    env.router.execute(env.bank.clone(), msg).unwrap();

    // bob pays 20 margin * 5x quote to get 9.0909090909 base
    // AMM after: 1100 : 90.9090909091, price: 12.1
    env.open_small_position(
        env.bob.clone(),
        Side::Buy,
        Uint128::from(10_000_000u64),
        Uint128::from(5_000_000u64),
        Some(Uint128::from(14_000_000u64)),
        Some(Uint128::from(8_000_000u64)),
        2u64,
    );

    // carol pays 10 margin * 5x quote to get 3.95256917 base
    // AMM after: 1150 : 86.9565217391
    env.open_small_position(
        env.carol.clone(),
        Side::Buy,
        Uint128::from(5_000_000u64),
        Uint128::from(5_000_000u64),
        Some(Uint128::from(15_000_000u64)),
        Some(Uint128::from(11_000_000u64)),
        2u64,
    );

    // alice pays 10 margin * 5x quote to get 3.6231884391 base
    // alice + carol base: 7.5757576091
    // AMM after: 1200 : 83.3333333, price: 14.4
    env.open_small_position(
        env.alice.clone(),
        Side::Buy,
        Uint128::from(5_000_000u64),
        Uint128::from(5_000_000u64),
        Some(Uint128::from(17_000_000u64)),
        Some(Uint128::from(13_000_000u64)),
        2u64,
    );

    // AMM after: 1100 : 90.9090909091, price: 12.1
    env.open_small_position(
        env.bob.clone(),
        Side::Sell,
        Uint128::from(10_000_000u64),
        Uint128::from(5_000_000u64),
        Some(Uint128::from(10_000_000u64)),
        Some(Uint128::from(15_000_000u64)),
        2u64,
    );

    let msg = env
        .vamm
        .set_fluctuation_limit_ratio(Uint128::from(38_000u128))
        .unwrap();
    env.router.execute(env.owner.clone(), msg).unwrap();

    let price = env.vamm.spot_price(&env.router.wrap()).unwrap();
    let msg = env
        .pricefeed
        .append_price("ETH".to_string(), price, timestamp)
        .unwrap();
    env.router.execute(env.owner.clone(), msg).unwrap();

    // half of alice's base asset: 3.6231884391 / 2 = 1.8115942196
    // AMM after: 1078.5079927008 : 92.7206851287, price: 11.6317949032
    // fluctuation: (12.1 - 11.63) / 12.1 = 0.03884297521
    // half of carol's base asset: 3.95256917 / 2 = 1.976284585
    // AMM after: 1055.9999998134 : 94.6969697137, price: 11.1513599961
    // fluctuation: (11.63 - 11.15) / 11.63 = 0.04127257094
    let msg = env
        .engine
        .liquidate(env.vamm.addr().to_string(), 5, Uint128::zero())
        .unwrap();
    env.router.execute(env.bob.clone(), msg).unwrap();

    let msg = env
        .engine
        .liquidate(env.vamm.addr().to_string(), 6, Uint128::zero())
        .unwrap();
    env.router.execute(env.bob.clone(), msg).unwrap();

    let msg = env
        .engine
        .liquidate(env.vamm.addr().to_string(), 3, Uint128::zero())
        .unwrap();
    let err = env.router.execute(env.alice.clone(), msg).unwrap_err();
    assert_eq!(
        StdError::GenericErr {
            msg: "partial liquidation failure - reply (id 5)".to_string()
        },
        err.downcast().unwrap()
    );
}
