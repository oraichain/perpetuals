use cosmwasm_std::{StdError, Uint128};
use margined_perp::margined_engine::Side;
use margined_utils::{
    cw_multi_test::Executor,
    testing::{to_decimals, SimpleScenario},
};

use crate::testing::new_simple_scenario;

#[test]
fn test_open_long_and_short_under_limit() {
    let SimpleScenario {
        mut router,
        alice,
        owner,
        engine,
        vamm,
        ..
    } = new_simple_scenario();

    let msg = vamm
        .set_base_asset_holding_cap(Uint128::from(10_000_000_000u128))
        .unwrap();
    router.execute(owner.clone(), msg).unwrap();

    let msg = engine
        .open_position(
            vamm.addr().to_string(),
            Side::Buy,
            to_decimals(110u64),
            to_decimals(1u64),
            Some(to_decimals(14)),
            Some(to_decimals(10)),
            to_decimals(0u64),
            vec![],
        )
        .unwrap();
    router.execute(alice.clone(), msg).unwrap();

    let msg = engine
        .open_position(
            vamm.addr().to_string(),
            Side::Sell,
            to_decimals(50u64),
            to_decimals(1u64),
            Some(to_decimals(7)),
            Some(to_decimals(16)),
            to_decimals(0u64),
            vec![],
        )
        .unwrap();
    router.execute(alice.clone(), msg).unwrap();
}

#[test]
fn test_open_two_long_positions_under_limit() {
    let SimpleScenario {
        mut router,
        alice,
        owner,
        engine,
        vamm,
        ..
    } = new_simple_scenario();

    let msg = vamm
        .set_base_asset_holding_cap(Uint128::from(10_000_000_000u128))
        .unwrap();
    router.execute(owner.clone(), msg).unwrap();

    let msg = engine
        .open_position(
            vamm.addr().to_string(),
            Side::Buy,
            to_decimals(55u64),
            to_decimals(1u64),
            Some(to_decimals(12)),
            Some(to_decimals(9)),
            to_decimals(0u64),
            vec![],
        )
        .unwrap();
    router.execute(alice.clone(), msg).unwrap();

    let msg = engine
        .open_position(
            vamm.addr().to_string(),
            Side::Buy,
            to_decimals(55u64),
            to_decimals(1u64),
            Some(to_decimals(14)),
            Some(to_decimals(8)),
            to_decimals(0u64),
            vec![],
        )
        .unwrap();
    router.execute(alice.clone(), msg).unwrap();
}

#[test]
fn test_open_short_and_long_under_limit() {
    let SimpleScenario {
        mut router,
        alice,
        owner,
        engine,
        vamm,
        ..
    } = new_simple_scenario();

    let msg = vamm
        .set_base_asset_holding_cap(Uint128::from(10_000_000_000u128))
        .unwrap();
    router.execute(owner.clone(), msg).unwrap();

    let msg = engine
        .open_position(
            vamm.addr().to_string(),
            Side::Sell,
            to_decimals(90u64),
            to_decimals(1u64),
            Some(to_decimals(8)),
            Some(to_decimals(12)),
            to_decimals(0u64),
            vec![],
        )
        .unwrap();
    router.execute(alice.clone(), msg).unwrap();

    let msg = engine
        .open_position(
            vamm.addr().to_string(),
            Side::Buy,
            to_decimals(50u64),
            to_decimals(1u64),
            Some(to_decimals(11)),
            Some(to_decimals(7)),
            to_decimals(0u64),
            vec![],
        )
        .unwrap();
    router.execute(alice.clone(), msg).unwrap();
}

#[test]
fn test_open_two_short_positions_under_limit() {
    let SimpleScenario {
        mut router,
        alice,
        owner,
        engine,
        vamm,
        ..
    } = new_simple_scenario();

    let msg = vamm
        .set_base_asset_holding_cap(Uint128::from(10_000_000_000u128))
        .unwrap();
    router.execute(owner.clone(), msg).unwrap();

    let msg = engine
        .open_position(
            vamm.addr().to_string(),
            Side::Sell,
            to_decimals(45u64),
            to_decimals(1u64),
            Some(to_decimals(8)),
            Some(to_decimals(12)),
            to_decimals(0u64),
            vec![],
        )
        .unwrap();
    router.execute(alice.clone(), msg).unwrap();

    let msg = engine
        .open_position(
            vamm.addr().to_string(),
            Side::Sell,
            to_decimals(45u64),
            to_decimals(1u64),
            Some(to_decimals(6)),
            Some(to_decimals(10)),
            to_decimals(0u64),
            vec![],
        )
        .unwrap();
    router.execute(alice.clone(), msg).unwrap();
}

#[test]
fn test_change_position_size_cap_and_open_position() {
    let SimpleScenario {
        mut router,
        alice,
        owner,
        engine,
        vamm,
        ..
    } = new_simple_scenario();

    let msg = vamm
        .set_base_asset_holding_cap(Uint128::from(10_000_000_000u128))
        .unwrap();
    router.execute(owner.clone(), msg).unwrap();

    let msg = engine
        .open_position(
            vamm.addr().to_string(),
            Side::Buy,
            to_decimals(25u64),
            to_decimals(1u64),
            Some(to_decimals(12)),
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

    let msg = vamm
        .set_base_asset_holding_cap(Uint128::from(20_000_000_000u128))
        .unwrap();
    router.execute(owner.clone(), msg).unwrap();
    let msg = engine
        .open_position(
            vamm.addr().to_string(),
            Side::Sell,
            to_decimals(16u64),
            to_decimals(10u64),
            Some(to_decimals(8)),
            Some(to_decimals(12)),
            to_decimals(0u64),
            vec![],
        )
        .unwrap();
    router.execute(alice.clone(), msg).unwrap();
}

#[test]
fn test_force_error_open_long_position_over_cap() {
    let SimpleScenario {
        mut router,
        alice,
        owner,
        engine,
        vamm,
        ..
    } = new_simple_scenario();

    let msg = vamm
        .set_base_asset_holding_cap(Uint128::from(10_000_000_000u128))
        .unwrap();
    router.execute(owner.clone(), msg).unwrap();

    let msg = engine
        .open_position(
            vamm.addr().to_string(),
            Side::Buy,
            to_decimals(120u64),
            to_decimals(1u64),
            Some(to_decimals(12)),
            Some(to_decimals(9)),
            to_decimals(0u64),
            vec![],
        )
        .unwrap();
    let err = router.execute(alice.clone(), msg).unwrap_err();
    assert_eq!(
        StdError::GenericErr {
            msg: "base asset holding exceeds cap".to_string(),
        },
        err.downcast().unwrap()
    );
}

#[test]
fn test_force_error_open_short_position_over_cap() {
    let SimpleScenario {
        mut router,
        alice,
        owner,
        engine,
        vamm,
        ..
    } = new_simple_scenario();

    let msg = vamm
        .set_base_asset_holding_cap(Uint128::from(10_000_000_000u128))
        .unwrap();
    router.execute(owner.clone(), msg).unwrap();

    let msg = engine
        .open_position(
            vamm.addr().to_string(),
            Side::Sell,
            to_decimals(95u64),
            to_decimals(1u64),
            Some(to_decimals(8)),
            Some(to_decimals(12)),
            to_decimals(0u64),
            vec![],
        )
        .unwrap();
    let err = router.execute(alice.clone(), msg).unwrap_err();
    assert_eq!(
        StdError::GenericErr {
            msg: "base asset holding exceeds cap".to_string(),
        },
        err.downcast().unwrap()
    );
}

#[test]
fn test_force_error_open_long_and_reverse_short_over_cap() {
    let SimpleScenario {
        mut router,
        alice,
        owner,
        engine,
        vamm,
        ..
    } = new_simple_scenario();

    let msg = vamm
        .set_base_asset_holding_cap(Uint128::from(10_000_000_000u128))
        .unwrap();
    router.execute(owner.clone(), msg).unwrap();

    let msg = engine
        .open_position(
            vamm.addr().to_string(),
            Side::Buy,
            to_decimals(10u64),
            to_decimals(1u64),
            Some(to_decimals(12)),
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
            to_decimals(20u64),
            to_decimals(10u64),
            Some(to_decimals(8)),
            Some(to_decimals(12)),
            to_decimals(0u64),
            vec![],
        )
        .unwrap();
    let err = router.execute(alice.clone(), msg).unwrap_err();
    assert_eq!(
        StdError::GenericErr {
            msg: "base asset holding exceeds cap".to_string(),
        },
        err.downcast().unwrap()
    );
}

#[test]
fn test_force_error_open_short_and_reverse_long_over_cap() {
    let SimpleScenario {
        mut router,
        alice,
        owner,
        engine,
        vamm,
        ..
    } = new_simple_scenario();

    let msg = vamm
        .set_base_asset_holding_cap(Uint128::from(10_000_000_000u128))
        .unwrap();
    router.execute(owner.clone(), msg).unwrap();

    let msg = engine
        .open_position(
            vamm.addr().to_string(),
            Side::Sell,
            to_decimals(9u64),
            to_decimals(1u64),
            Some(to_decimals(8)),
            Some(to_decimals(12)),
            to_decimals(0u64),
            vec![],
        )
        .unwrap();
    router.execute(alice.clone(), msg).unwrap();

    let msg = engine
        .open_position(
            vamm.addr().to_string(),
            Side::Buy,
            to_decimals(21u64),
            to_decimals(10u64),
            Some(to_decimals(12)),
            Some(to_decimals(7)),
            to_decimals(0u64),
            vec![],
        )
        .unwrap();
    let err = router.execute(alice.clone(), msg).unwrap_err();
    assert_eq!(
        StdError::GenericErr {
            msg: "base asset holding exceeds cap".to_string(),
        },
        err.downcast().unwrap()
    );
}
