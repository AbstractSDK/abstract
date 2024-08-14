use crate::state::{Config, CONFIG};
use abstract_ica::{msg::ConfigResponse, ChainType, IcaAction, IcaActionResponse};
use abstract_std::objects::TruncatedChainId;
use cosmwasm_std::{ensure_eq, CosmosMsg, Deps, Env};

use crate::{chain_types::evm, contract::IcaClientResult, error::IcaClientError};

/// Timeout in seconds
pub const PACKET_LIFETIME: u64 = 60 * 60;

pub fn config(deps: Deps) -> IcaClientResult<ConfigResponse> {
    let Config {
        version_control,
        ans_host,
    } = CONFIG.load(deps.storage)?;
    Ok(ConfigResponse {
        ans_host: ans_host.address.into_string(),
        version_control_address: version_control.address.into_string(),
    })
}

pub(crate) fn ica_action(
    deps: Deps,
    env: Env,
    _proxy_address: String,
    chain: TruncatedChainId,
    mut actions: Vec<IcaAction>,
) -> IcaClientResult<IcaActionResponse> {
    // match chain-id with cosmos or EVM
    use abstract_ica::CastChainType;
    let chain_type = chain.chain_type().ok_or(IcaClientError::NoChainType {
        chain: chain.to_string(),
    })?;

    // todo: what do we do for msgs that contain both cosmos and EVM messages?
    // Best to err if there's conflict.

    // sort actions
    // 1) Transfers
    // 2) Calls
    // 3) Queries
    actions.sort_unstable();

    let cfg = CONFIG.load(deps.storage)?;

    let process_action = |action: IcaAction| -> IcaClientResult<Vec<CosmosMsg>> {
        match action {
            IcaAction::Execute(ica_exec) => match ica_exec {
                abstract_ica::IcaExecute::Evm { msgs, callback } => {
                    ensure_eq!(
                        chain_type,
                        ChainType::Evm,
                        IcaClientError::WrongChainType {
                            chain: chain.to_string(),
                            ty: chain_type.to_string()
                        }
                    );

                    let msg = evm::execute(&deps.querier, &cfg.version_control, msgs, callback)?;

                    Ok(vec![msg.into()])
                }
                _ => unimplemented!(),
            },
            IcaAction::Fund {
                funds,
                receiver,
                memo,
            } => match chain_type {
                ChainType::Evm => Ok(vec![evm::send_funds(
                    deps, &env, &chain, &cfg, funds, receiver, memo,
                )?]),
                _ => unimplemented!(),
            },
            _ => unimplemented!(),
        }
    };

    // TODO: can we use `flat_map` here?
    let maybe_msgs: Result<Vec<Vec<CosmosMsg>>, _> =
        actions.into_iter().map(process_action).collect();
    let msgs = maybe_msgs?.into_iter().flatten().collect();

    Ok(IcaActionResponse { msgs })
}

#[cfg(test)]
mod tests {
    use std::str::FromStr as _;

    use super::*;

    use crate::test_common::mock_init;
    use abstract_std::{
        objects::{
            module::{Module, ModuleInfo},
            module_reference::ModuleReference,
            ChannelEntry, ContractEntry,
        },
        version_control::{self as vc, ModuleConfiguration},
    };
    use abstract_testing::prelude::{TEST_VERSION_CONTROL, *};
    use cosmwasm_std::{
        from_json,
        testing::{mock_dependencies, mock_env},
        Addr, HexBinary,
    };

    use evm::types;
    use polytone_evm::EVM_NOTE_ID;
    use speculoos::prelude::*;

    type IbcClientTestResult = Result<(), IcaClientError>;

    const EVM_CHAIN: &str = "bartio";
    const COSMOS_CHAIN: &str = "juno";

    fn env_note_addr() -> Addr {
        Addr::unchecked("evm_note_addr".to_string())
    }

    fn ucs_forwarder_addr() -> Addr {
        Addr::unchecked("ucs_forwarder".to_string())
    }

    /// setup the querier with the proper responses and state
    fn state_setup() -> MockQuerierBuilder {
        let chain_name = TruncatedChainId::from_str(EVM_CHAIN).unwrap();

        mocked_account_querier_builder()
            .contracts(vec![(
                &ContractEntry {
                    contract: types::UCS01_FORWARDER_CONTRACT.to_string(),
                    protocol: types::UCS01_PROTOCOL.to_string(),
                },
                ucs_forwarder_addr(),
            )])
            .channels(vec![(
                &ChannelEntry {
                    connected_chain: chain_name.clone(),
                    protocol: types::UCS01_PROTOCOL.to_string(),
                },
                "channel-1".into(),
            )])
            .builder()
            .with_smart_handler(env_note_addr().as_str(), |bin| {
                let msg = from_json::<evm_note::msg::QueryMsg>(bin).unwrap();
                match msg {
                    evm_note::msg::QueryMsg::RemoteAddress { .. } => {
                        to_json_binary(&evm_note::msg::RemoteAddressResponse {
                            remote_address: Some("123fff".to_owned()),
                        })
                        .map_err(|e| e.to_string())
                    }
                    _ => panic!("should only query for RemoteAddress"),
                }
            })
            .with_smart_handler(TEST_VERSION_CONTROL, |bin| {
                let msg = from_json::<vc::QueryMsg>(bin).unwrap();
                match msg {
                    vc::QueryMsg::Modules { infos } => {
                        assert_eq!(
                            infos[0],
                            ModuleInfo::from_id(
                                EVM_NOTE_ID,
                                abstract_ica::POLYTONE_EVM_VERSION.parse().unwrap()
                            )
                            .unwrap()
                        );
                        to_json_binary(&vc::ModulesResponse {
                            modules: vec![vc::ModuleResponse {
                                config: ModuleConfiguration::default(),
                                module: Module {
                                    info: ModuleInfo::from_id(
                                        EVM_NOTE_ID,
                                        abstract_ica::POLYTONE_EVM_VERSION.parse().unwrap(),
                                    )
                                    .unwrap(),
                                    reference: ModuleReference::Native(Addr::unchecked(
                                        env_note_addr().to_string(),
                                    )),
                                },
                            }],
                        })
                        .map_err(|e| e.to_string())
                    }
                    _ => panic!("should only query for Polytone module"),
                }
            })
    }

    mod ica_action {
        use crate::contract::query;

        use super::*;
        use std::str::FromStr;

        use abstract_ica::msg::QueryMsg;
        use abstract_std::objects::TruncatedChainId;

        use cosmwasm_std::{coins, wasm_execute};
        use evm::types;
        use evm_note::msg::EvmMsg;

        use types::Ucs01ForwarderExecuteMsg;

        #[test]
        fn config() -> IbcClientTestResult {
            let mut deps = mock_dependencies();

            deps.querier = state_setup().build();

            mock_init(deps.as_mut())?;
            let res = query(deps.as_ref(), mock_env(), QueryMsg::Config {})?;
            let res: ConfigResponse = from_json(&res).unwrap();
            assert_eq!(
                res,
                ConfigResponse {
                    ans_host: TEST_ANS_HOST.to_owned(),
                    version_control_address: TEST_VERSION_CONTROL.to_owned()
                }
            );
            Ok(())
        }

        #[test]
        fn evm_exec_no_callback() -> IbcClientTestResult {
            let mut deps = mock_dependencies();
            let chain_name = TruncatedChainId::from_str(EVM_CHAIN)?;

            deps.querier = state_setup().build();

            mock_init(deps.as_mut())?;

            let msg = QueryMsg::IcaAction {
                proxy_address: TEST_PROXY.into(),
                chain: chain_name,
                actions: vec![IcaAction::Execute(abstract_ica::IcaExecute::Evm {
                    msgs: vec![EvmMsg::Call {
                        to: "to".to_string(),
                        data: HexBinary::from(vec![0x01]),
                    }],
                    callback: None,
                })],
            };

            let res = query(deps.as_ref(), mock_env(), msg)?;
            let res: IcaActionResponse = from_json(&res).unwrap();

            assert_that!(res).is_equal_to(IcaActionResponse {
                msgs: vec![CosmosMsg::Wasm(wasm_execute(
                    env_note_addr(),
                    &evm_note::msg::ExecuteMsg::Execute {
                        callback: None,
                        msgs: vec![EvmMsg::Call {
                            to: "to".to_string(),
                            data: HexBinary::from(vec![0x01]),
                        }],
                        timeout_seconds: PACKET_LIFETIME.into(),
                    },
                    vec![],
                )?)],
            });

            Ok(())
        }

        #[test]
        fn evm_fund_no_callback() -> IbcClientTestResult {
            use super::*;

            let mut deps = mock_dependencies();
            let chain_name = TruncatedChainId::from_str(EVM_CHAIN)?;

            deps.querier = state_setup().build();

            mock_init(deps.as_mut())?;

            let receiver = HexBinary::from_hex("123fff").unwrap();

            let msg = QueryMsg::IcaAction {
                proxy_address: TEST_PROXY.into(),
                chain: chain_name,
                actions: vec![IcaAction::Fund {
                    funds: coins(1, "test"),
                    receiver: Some(receiver.clone().into()),
                    memo: None,
                }],
            };

            let res = query(deps.as_ref(), mock_env(), msg)?;
            let res: IcaActionResponse = from_json(&res).unwrap();

            assert_that!(res).is_equal_to(IcaActionResponse {
                msgs: vec![CosmosMsg::Wasm(wasm_execute(
                    ucs_forwarder_addr(),
                    &Ucs01ForwarderExecuteMsg::Transfer {
                        channel: "channel-1".into(),
                        receiver,
                        memo: "".to_string(),
                        timeout: PACKET_LIFETIME.into(),
                    },
                    coins(1, "test"),
                )?)],
            });

            Ok(())
        }

        #[test]
        fn evm_fund_no_receiver() -> IbcClientTestResult {
            use super::*;

            let mut deps = mock_dependencies();
            let chain_name = TruncatedChainId::from_str(EVM_CHAIN)?;

            deps.querier = state_setup().build();

            mock_init(deps.as_mut())?;

            let msg = QueryMsg::IcaAction {
                proxy_address: TEST_PROXY.into(),
                chain: chain_name,
                actions: vec![IcaAction::Fund {
                    funds: coins(1, "test"),
                    receiver: None,
                    memo: None,
                }],
            };

            let res = query(deps.as_ref(), mock_env(), msg)?;
            let res: IcaActionResponse = from_json(&res).unwrap();

            assert_that!(res).is_equal_to(IcaActionResponse {
                msgs: vec![CosmosMsg::Wasm(wasm_execute(
                    ucs_forwarder_addr(),
                    &Ucs01ForwarderExecuteMsg::Transfer {
                        channel: "channel-1".into(),
                        receiver: HexBinary::from_hex("123fff").unwrap(),
                        memo: "".to_string(),
                        timeout: PACKET_LIFETIME.into(),
                    },
                    coins(1, "test"),
                )?)],
            });

            Ok(())
        }

        #[test]
        fn evm_exec_non_evm_chaintype() -> IbcClientTestResult {
            let mut deps = mock_dependencies();
            let chain_name = TruncatedChainId::from_str(COSMOS_CHAIN)?;

            deps.querier = state_setup().build();

            mock_init(deps.as_mut())?;

            let msg = QueryMsg::IcaAction {
                proxy_address: TEST_PROXY.into(),
                chain: chain_name.clone(),
                actions: vec![IcaAction::Execute(abstract_ica::IcaExecute::Evm {
                    msgs: vec![EvmMsg::Call {
                        to: "to".to_string(),
                        data: HexBinary::from(vec![0x01]),
                    }],
                    callback: None,
                })],
            };

            let err = query(deps.as_ref(), mock_env(), msg).unwrap_err();
            assert_eq!(
                err,
                IcaClientError::WrongChainType {
                    chain: chain_name.to_string(),
                    ty: ChainType::Cosmos.to_string()
                }
            );

            Ok(())
        }
    }
}
