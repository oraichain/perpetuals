use crate::state::{Config, State, UserStake};

use cosmwasm_std::{Addr, Timestamp, Uint128};
use margined_common::asset::{AssetInfo, NATIVE_DENOM};
use margined_perp::margined_staking::{ExecuteMsg, InstantiateMsg, QueryMsg, TotalStakedResponse};
use margined_utils::testing::test_tube::{TestTubeScenario, STAKING_CONTRACT_BYTES};
use osmosis_test_tube::{
    cosmrs::proto::cosmos::{bank::v1beta1::MsgSend, base::v1beta1::Coin},
    Account, Bank, Module, Wasm,
};

#[test]
fn test_query_config() {
    let TestTubeScenario {
        router,
        accounts,
        fee_pool,
        ..
    } = TestTubeScenario::default();

    let signer = &accounts[0];

    let wasm = Wasm::new(&router);

    let staking_code_id = wasm
        .store_code(STAKING_CONTRACT_BYTES, None, signer)
        .unwrap()
        .data
        .code_id;

    let staking_address = wasm
        .instantiate(
            staking_code_id,
            &InstantiateMsg {
                fee_pool: fee_pool.addr().to_string(),
                deposit_token: AssetInfo::NativeToken {
                    denom: NATIVE_DENOM.to_string(),
                },
                reward_token: AssetInfo::NativeToken {
                    denom: NATIVE_DENOM.to_string(),
                },
                // deposit_token: AssetInfo::Token {
                //     contract_addr: usdc.addr(),
                // },
                // reward_token: AssetInfo::Token {
                //     contract_addr: usdc.addr(),
                // }, // should be ORAIX
                tokens_per_interval: 1_000_000u128.into(),
            },
            None,
            Some("margined-staking"),
            &[],
            signer,
        )
        .unwrap()
        .data
        .address;

    let config: Config = wasm.query(&staking_address, &QueryMsg::Config {}).unwrap();
    assert_eq!(
        config,
        Config {
            fee_pool: fee_pool.addr(),
            deposit_token: AssetInfo::NativeToken {
                denom: NATIVE_DENOM.to_string(),
            },
            reward_token: AssetInfo::NativeToken {
                denom: NATIVE_DENOM.to_string(),
            },
            tokens_per_interval: 1_000_000u128.into(),
        }
    );
}

#[test]
fn test_query_state() {
    let TestTubeScenario {
        router,
        accounts,
        fee_pool,
        ..
    } = TestTubeScenario::default();

    let signer = &accounts[0];

    let wasm = Wasm::new(&router);

    let staking_code_id = wasm
        .store_code(STAKING_CONTRACT_BYTES, None, signer)
        .unwrap()
        .data
        .code_id;

    let staking_address = wasm
        .instantiate(
            staking_code_id,
            &InstantiateMsg {
                fee_pool: fee_pool.addr().to_string(),
                deposit_token: AssetInfo::NativeToken {
                    denom: NATIVE_DENOM.to_string(),
                },
                reward_token: AssetInfo::NativeToken {
                    denom: NATIVE_DENOM.to_string(),
                },
                // deposit_token: AssetInfo::Token {
                //     contract_addr: usdc.addr(),
                // },
                // reward_token: AssetInfo::Token {
                //     contract_addr: usdc.addr(),
                // }, // should be ORAIX
                tokens_per_interval: 1_000_000u128.into(),
            },
            None,
            Some("margined-staking"),
            &[],
            signer,
        )
        .unwrap()
        .data
        .address;

    let state: State = wasm.query(&staking_address, &QueryMsg::State {}).unwrap();
    assert_eq!(
        state,
        State {
            is_open: false,
            last_distribution: Timestamp::from_nanos(router.get_block_time_nanos() as u64),
        }
    );
}

#[test]
fn test_query_owner() {
    let TestTubeScenario {
        router,
        accounts,
        fee_pool,
        ..
    } = TestTubeScenario::default();

    let signer = &accounts[0];

    let wasm = Wasm::new(&router);

    let staking_code_id = wasm
        .store_code(STAKING_CONTRACT_BYTES, None, signer)
        .unwrap()
        .data
        .code_id;

    let staking_address = wasm
        .instantiate(
            staking_code_id,
            &InstantiateMsg {
                fee_pool: fee_pool.addr().to_string(),
                deposit_token: AssetInfo::NativeToken {
                    denom: NATIVE_DENOM.to_string(),
                },
                reward_token: AssetInfo::NativeToken {
                    denom: NATIVE_DENOM.to_string(),
                },
                // deposit_token: AssetInfo::Token {
                //     contract_addr: usdc.addr(),
                // },
                // reward_token: AssetInfo::Token {
                //     contract_addr: usdc.addr(),
                // }, // should be ORAIX
                tokens_per_interval: 1_000_000u128.into(),
            },
            None,
            Some("margined-staking"),
            &[],
            signer,
        )
        .unwrap()
        .data
        .address;

    let owner: Addr = wasm.query(&staking_address, &QueryMsg::Owner {}).unwrap();
    assert_eq!(owner, Addr::unchecked(signer.address()));
}

#[test]
fn test_query_get_claimable() {
    let TestTubeScenario {
        router,
        accounts,
        fee_pool,
        ..
    } = TestTubeScenario::default();

    let signer = &accounts[0];

    let wasm = Wasm::new(&router);

    let staking_code_id = wasm
        .store_code(STAKING_CONTRACT_BYTES, None, signer)
        .unwrap()
        .data
        .code_id;

    let staking_address = wasm
        .instantiate(
            staking_code_id,
            &InstantiateMsg {
                fee_pool: fee_pool.addr().to_string(),
                deposit_token: AssetInfo::NativeToken {
                    denom: NATIVE_DENOM.to_string(),
                },
                reward_token: AssetInfo::NativeToken {
                    denom: NATIVE_DENOM.to_string(),
                },
                // deposit_token: AssetInfo::Token {
                //     contract_addr: usdc.addr(),
                // },
                // reward_token: AssetInfo::Token {
                //     contract_addr: usdc.addr(),
                // }, // should be ORAIX
                tokens_per_interval: 1_000_000u128.into(),
            },
            None,
            Some("margined-staking"),
            &[],
            signer,
        )
        .unwrap()
        .data
        .address;

    let bank = Bank::new(&router);

    bank.send(
        MsgSend {
            from_address: signer.address(),
            to_address: staking_address.clone(),
            amount: [Coin {
                amount: 10u128.to_string(),
                denom: NATIVE_DENOM.to_string(),
            }]
            .to_vec(),
        },
        &signer,
    )
    .unwrap();

    let amount: Uint128 = wasm
        .query(
            &staking_address,
            &QueryMsg::GetClaimable {
                user: accounts[0].address(),
            },
        )
        .unwrap();
    assert_eq!(amount, Uint128::zero());

    wasm.execute(&staking_address, &ExecuteMsg::Unpause {}, &[], &signer)
        .unwrap();

    let amount_to_stake = 1_000_000u128;
    wasm.execute(
        &staking_address,
        &ExecuteMsg::Stake {},
        &[Coin {
            amount: amount_to_stake.to_string(),
            denom: NATIVE_DENOM.to_string(),
        }],
        &accounts[0],
    )
    .unwrap();

    router.increase_time(5u64);

    let amount: Uint128 = wasm
        .query(
            &staking_address,
            &QueryMsg::GetClaimable {
                user: accounts[0].address(),
            },
        )
        .unwrap();
    assert_eq!(amount, Uint128::from(5_000_000u128));
}

#[test]
fn test_query_get_user_staked_amount() {
    let TestTubeScenario {
        router,
        accounts,
        fee_pool,
        ..
    } = TestTubeScenario::default();

    let signer = &accounts[0];

    let wasm = Wasm::new(&router);

    let staking_code_id = wasm
        .store_code(STAKING_CONTRACT_BYTES, None, signer)
        .unwrap()
        .data
        .code_id;

    let staking_address = wasm
        .instantiate(
            staking_code_id,
            &InstantiateMsg {
                fee_pool: fee_pool.addr().to_string(),
                deposit_token: AssetInfo::NativeToken {
                    denom: NATIVE_DENOM.to_string(),
                },
                reward_token: AssetInfo::NativeToken {
                    denom: NATIVE_DENOM.to_string(),
                },
                // deposit_token: AssetInfo::Token {
                //     contract_addr: usdc.addr(),
                // },
                // reward_token: AssetInfo::Token {
                //     contract_addr: usdc.addr(),
                // }, // should be ORAIX
                tokens_per_interval: 1_000_000u128.into(),
            },
            None,
            Some("margined-staking"),
            &[],
            signer,
        )
        .unwrap()
        .data
        .address;

    let bank = Bank::new(&router);

    bank.send(
        MsgSend {
            from_address: signer.address(),
            to_address: fee_pool.0.to_string(),
            amount: [Coin {
                amount: 1_000_000_000u128.to_string(),
                denom: NATIVE_DENOM.to_string(),
            }]
            .to_vec(),
        },
        &signer,
    )
    .unwrap();

    let amount: UserStake = wasm
        .query(
            &staking_address,
            &QueryMsg::GetUserStakedAmount {
                user: accounts[0].address(),
            },
        )
        .unwrap();
    assert_eq!(amount, UserStake::default());

    wasm.execute(&staking_address, &ExecuteMsg::Unpause {}, &[], &signer)
        .unwrap();

    let _res = wasm
        .execute(
            fee_pool.0.as_str(),
            &margined_perp::margined_fee_pool::ExecuteMsg::AddToken {
                token: NATIVE_DENOM.to_string(),
            },
            &[],
            &signer,
        )
        .unwrap();

    // change owner of fee pool to staking contract
    let _res = wasm
        .execute(
            fee_pool.0.as_str(),
            &margined_perp::margined_fee_pool::ExecuteMsg::UpdateOwner {
                owner: staking_address.clone(),
            },
            &[],
            &signer,
        )
        .unwrap();

    let amount_to_stake = 1_000_000u128;
    wasm.execute(
        &staking_address,
        &ExecuteMsg::Stake {},
        &[Coin {
            amount: amount_to_stake.to_string(),
            denom: NATIVE_DENOM.to_string(),
        }],
        &accounts[0],
    )
    .unwrap();

    router.increase_time(5u64);

    let amount: UserStake = wasm
        .query(
            &staking_address,
            &QueryMsg::GetUserStakedAmount {
                user: accounts[0].address(),
            },
        )
        .unwrap();
    assert_eq!(
        amount,
        UserStake {
            staked_amounts: amount_to_stake.into(),
            previous_cumulative_rewards_per_token: Uint128::zero(),
            claimable_rewards: Uint128::zero(),
            cumulative_rewards: Uint128::zero(),
        }
    );

    wasm.execute(
        &staking_address,
        &ExecuteMsg::Claim { recipient: None },
        &[],
        &accounts[0],
    )
    .unwrap();

    let amount: UserStake = wasm
        .query(
            &staking_address,
            &QueryMsg::GetUserStakedAmount {
                user: accounts[0].address(),
            },
        )
        .unwrap();
    assert_eq!(
        amount,
        UserStake {
            staked_amounts: amount_to_stake.into(),
            previous_cumulative_rewards_per_token: Uint128::from(10_000_000u128),
            claimable_rewards: Uint128::zero(),
            cumulative_rewards: Uint128::from(10_000_000u128),
        }
    );
}

#[test]
fn test_query_get_total_staked_amount() {
    let TestTubeScenario {
        router,
        accounts,
        fee_pool,
        ..
    } = TestTubeScenario::default();

    let signer = &accounts[0];

    let wasm = Wasm::new(&router);

    let staking_code_id = wasm
        .store_code(STAKING_CONTRACT_BYTES, None, signer)
        .unwrap()
        .data
        .code_id;

    let staking_address = wasm
        .instantiate(
            staking_code_id,
            &InstantiateMsg {
                fee_pool: fee_pool.addr().to_string(),
                deposit_token: AssetInfo::NativeToken {
                    denom: NATIVE_DENOM.to_string(),
                },
                reward_token: AssetInfo::NativeToken {
                    denom: NATIVE_DENOM.to_string(),
                },
                // deposit_token: AssetInfo::Token {
                //     contract_addr: usdc.addr(),
                // },
                // reward_token: AssetInfo::Token {
                //     contract_addr: usdc.addr(),
                // }, // should be ORAIX
                tokens_per_interval: 1_000_000u128.into(),
            },
            None,
            Some("margined-staking"),
            &[],
            signer,
        )
        .unwrap()
        .data
        .address;

    let bank = Bank::new(&router);

    bank.send(
        MsgSend {
            from_address: signer.address(),
            to_address: fee_pool.0.to_string(),
            amount: [Coin {
                amount: 1_000_000_000u128.to_string(),
                denom: NATIVE_DENOM.to_string(),
            }]
            .to_vec(),
        },
        &signer,
    )
    .unwrap();

    let res: TotalStakedResponse = wasm
        .query(&staking_address, &QueryMsg::GetTotalStakedAmount {})
        .unwrap();
    assert_eq!(res.amount, Uint128::zero());

    wasm.execute(&staking_address, &ExecuteMsg::Unpause {}, &[], &signer)
        .unwrap();

    let _res = wasm
        .execute(
            fee_pool.0.as_str(),
            &margined_perp::margined_fee_pool::ExecuteMsg::AddToken {
                token: NATIVE_DENOM.to_string(),
            },
            &[],
            &signer,
        )
        .unwrap();

    // change owner of fee pool to staking contract
    let _res = wasm
        .execute(
            fee_pool.0.as_str(),
            &margined_perp::margined_fee_pool::ExecuteMsg::UpdateOwner {
                owner: staking_address.clone(),
            },
            &[],
            &signer,
        )
        .unwrap();

    let amount_to_stake = 1_000_000u128;
    wasm.execute(
        &staking_address,
        &ExecuteMsg::Stake {},
        &[Coin {
            amount: amount_to_stake.to_string(),
            denom: NATIVE_DENOM.to_string(),
        }],
        &accounts[0],
    )
    .unwrap();

    router.increase_time(5u64);

    let res: TotalStakedResponse = wasm
        .query(&staking_address, &QueryMsg::GetTotalStakedAmount {})
        .unwrap();
    assert_eq!(res.amount, Uint128::from(amount_to_stake));

    let amount_to_unstake = 500_000u128;
    wasm.execute(
        &staking_address,
        &ExecuteMsg::Unstake {
            amount: amount_to_unstake.into(),
        },
        &[],
        &accounts[0],
    )
    .unwrap();

    let res: TotalStakedResponse = wasm
        .query(&staking_address, &QueryMsg::GetTotalStakedAmount {})
        .unwrap();
    assert_eq!(
        res.amount,
        Uint128::from(amount_to_stake - amount_to_unstake)
    );
}
