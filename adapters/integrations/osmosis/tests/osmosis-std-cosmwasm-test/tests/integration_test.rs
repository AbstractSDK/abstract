mod helpers;
use core::time;
use std::time::{SystemTime, UNIX_EPOCH};

use cosmwasm_std::Coin;
use helpers::with_env_setup;
use osmosis_std::{
    shim::{Duration, Timestamp},
    types::osmosis::{
        epochs::v1beta1::EpochInfo,
        gamm::{
            self,
            v1beta1::{MsgSwapExactAmountInResponse, Pool, PoolParams},
        },
        poolmanager::v1beta1::SwapAmountInRoute,
        twap::v1beta1::GeometricTwapToNowRequest,
    },
};
use osmosis_std_cosmwasm_test::msg::{
    ArithmeticTwapToNowRequest, ArithmeticTwapToNowResponse, ExecuteMsg,
    GeometricTwapToNowResponse, QueryEpochsInfoResponse, QueryMapResponse, QueryMsg,
    QueryNumPoolsResponse, QueryPoolParamsResponse, QueryPoolResponse,
};
use osmosis_test_tube::RunnerError::ExecuteError;
use osmosis_test_tube::{Account, Runner};

#[test]
fn test_u64_response_deser() {
    with_env_setup(
        |_app, wasm, _signer, _code_id, contract_addr| {
            assert_eq!(
                QueryNumPoolsResponse { num_pools: 0 },
                wasm.query(&contract_addr, &QueryMsg::QueryNumPools {})
                    .unwrap()
            )
        },
        false,
    )
}

#[test]
fn test_bool_response_deser() {
    with_env_setup(
        |_app, wasm, _signer, _code_id, contract_addr| {
            let res: QueryEpochsInfoResponse = wasm
                .query(&contract_addr, &QueryMsg::QueryEpochsInfo {})
                .unwrap();
            let day = &res.epochs[0];

            let EpochInfo {
                identifier,
                start_time: _,
                duration: _,
                current_epoch,
                current_epoch_start_time: _,
                epoch_counting_started,
                current_epoch_start_height,
            } = day;

            assert_eq!(identifier, "day");
            assert_eq!(current_epoch, &4);
            assert_eq!(epoch_counting_started, &true);
            assert_eq!(current_epoch_start_height, &4);
        },
        false,
    )
}

#[test]
fn test_timestamp_response_deser() {
    with_env_setup(
        |_app, wasm, _signer, _code_id, contract_addr| {
            let res: QueryEpochsInfoResponse = wasm
                .query(&contract_addr, &QueryMsg::QueryEpochsInfo {})
                .unwrap();
            let day = &res.epochs[0];

            let EpochInfo {
                identifier: _,
                start_time,
                duration: _,
                current_epoch: _,
                current_epoch_start_time,
                epoch_counting_started: _,
                current_epoch_start_height: _,
            } = day;

            assert_eq!(
                // 0001-01-01T00:00:00Z
                start_time.as_ref().unwrap().to_owned(),
                Timestamp {
                    seconds: -62135596800,
                    nanos: 0
                }
            );

            assert_eq!(
                // 0001-01-04T00:00:00Z (+3 days from start_time)
                current_epoch_start_time.as_ref().unwrap().to_owned(),
                Timestamp {
                    seconds: -62135596800 + (3 * 24 * 60 * 60),
                    nanos: 0
                }
            );
        },
        false,
    )
}

#[test]
fn test_duration_response_deser() {
    with_env_setup(
        |_app, wasm, _signer, _code_id, contract_addr| {
            let res: QueryEpochsInfoResponse = wasm
                .query(&contract_addr, &QueryMsg::QueryEpochsInfo {})
                .unwrap();
            let day = &res.epochs[0];

            let EpochInfo {
                identifier: _,
                start_time: _,
                duration,
                current_epoch: _,
                current_epoch_start_time: _,
                epoch_counting_started: _,
                current_epoch_start_height: _,
            } = day;

            assert_eq!(
                duration.as_ref().unwrap().to_owned(),
                Duration {
                    seconds: 86400,
                    nanos: 0
                }
            );
        },
        false,
    )
}

#[test]
fn test_any_balancer_pool_response_deser() {
    with_env_setup(
        |app, wasm, signer, _code_id, contract_addr| {
            let pools = helpers::setup_pools(app, &signer);
            let pool_id = pools[0];

            let res: QueryPoolResponse = wasm
                .query(&contract_addr, &QueryMsg::QueryPool { pool_id })
                .unwrap();

            let pool: Pool = res.pool.unwrap().try_into().unwrap();
            assert_eq!(pool, helpers::mock_balancner_pool());
        },
        false,
    );
}

#[test]
fn test_any_balancer_pool_params_response_deser() {
    with_env_setup(
        |app, wasm, signer, _code_id, contract_addr| {
            let pools = helpers::setup_pools(app, &signer);
            let pool_id = pools[0];

            let res: QueryPoolParamsResponse = wasm
                .query(&contract_addr, &QueryMsg::QueryPoolParams { pool_id })
                .unwrap();

            let pool: PoolParams = res.params.unwrap().try_into().unwrap();

            assert_eq!(pool, helpers::mock_balancner_pool().pool_params.unwrap());
        },
        false,
    );
}

#[test]
fn test_twap_query() {
    with_env_setup(
        |app, wasm, signer, _code_id, contract_addr| {
            let pools = helpers::setup_pools(app, &signer);
            let pool_id = pools[0];

            let swap = || {
                app.execute::<_, MsgSwapExactAmountInResponse>(
                    gamm::v1beta1::MsgSwapExactAmountIn {
                        sender: signer.address(),
                        routes: vec![SwapAmountInRoute {
                            pool_id,
                            token_out_denom: "uion".to_string(),
                        }],
                        token_in: Some(Coin::new(30, "uosmo").into()),
                        token_out_min_amount: "1".to_string(),
                    },
                    gamm::v1beta1::MsgSwapExactAmountIn::TYPE_URL,
                    &signer,
                )
                .unwrap()
            };

            swap();
            swap();

            let time = SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .unwrap()
                .checked_add(time::Duration::from_secs(30))
                .unwrap();

            let res: ArithmeticTwapToNowResponse = wasm
                .query(
                    &contract_addr,
                    &QueryMsg::QueryArithmeticTwapToNow(ArithmeticTwapToNowRequest {
                        pool_id,
                        base_asset: "uosmo".to_string(),
                        quote_asset: "uion".to_string(),
                        start_time: Some(Timestamp {
                            seconds: time.as_secs() as i64,
                            nanos: 0,
                        }),
                    }),
                )
                .unwrap();

            assert_eq!(
                res.arithmetic_twap.chars().take(4).collect::<String>(),
                "0.94"
            );

            let res: GeometricTwapToNowResponse = wasm
                .query(
                    &contract_addr,
                    &QueryMsg::QueryGeometricTwapToNow(GeometricTwapToNowRequest {
                        pool_id,
                        base_asset: "uosmo".to_string(),
                        quote_asset: "uion".to_string(),
                        start_time: Some(Timestamp {
                            seconds: time.as_secs() as i64,
                            nanos: 0,
                        }),
                    }),
                )
                .unwrap();

            assert_eq!(
                res.geometric_twap.chars().take(4).collect::<String>(),
                "0.94"
            );
        },
        true,
    );
}

#[test]
fn test_cosmwasm_vm_storage_plus_compatability() {
    with_env_setup(
        |_app, wasm, signer, _code_id, contract_addr| {
            wasm.execute(
                &contract_addr,
                &ExecuteMsg::SetMap {
                    key: "key".to_string(),
                    value: "value".to_string(),
                },
                &[],
                &signer,
            )
            .unwrap();

            let res: QueryMapResponse = wasm
                .query(
                    &contract_addr,
                    &QueryMsg::QueryMap {
                        key: "key".to_string(),
                    },
                )
                .unwrap();

            assert_eq!(res.value, "value".to_string());
        },
        true,
    );
}

#[test]
fn test_execute_after_error_should_not_messed_up_block_height() {
    with_env_setup(
        |app, wasm, owner, _code_id, contract_addr| {
            let non_owner = app.init_account(&[]).unwrap();
            let err = wasm
                .execute(
                    &contract_addr,
                    &ExecuteMsg::SetMap {
                        key: "key".to_string(),
                        value: "value".to_string(),
                    },
                    &[],
                    &non_owner,
                )
                .unwrap_err();

            assert_eq!(err, ExecuteError {
                msg: "failed to execute message; message index: 0: Unauthorized: execute wasm contract failed".to_string(),
            });

            // after error, end block should be called properly so that height is updated
            wasm.execute(
                &contract_addr,
                &ExecuteMsg::SetMap {
                    key: "key".to_string(),
                    value: "value".to_string(),
                },
                &[],
                &owner,
            )
            .unwrap();

            let res: QueryMapResponse = wasm
                .query(
                    &contract_addr,
                    &QueryMsg::QueryMap {
                        key: "key".to_string(),
                    },
                )
                .unwrap();

            assert_eq!(res.value, "value".to_string());
        },
        true,
    );
}
