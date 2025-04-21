use abstract_ica::{msg::ConfigResponse, ChainType, IcaAction, IcaActionResponse};
use abstract_sdk::feature_objects::{AnsHost, RegistryContract};
use abstract_std::{native_addrs, objects::TruncatedChainId};
use cosmwasm_std::{ensure_eq, CosmosMsg, Deps, Env};

use crate::{chain_types::evm, contract::IcaClientResult, error::IcaClientError};

pub fn config(deps: Deps, env: &Env) -> IcaClientResult<ConfigResponse> {
    let abstract_code_id =
        native_addrs::abstract_code_id(&deps.querier, env.contract.address.clone())?;

    Ok(ConfigResponse {
        ans_host: AnsHost::new(deps, abstract_code_id)?.address,
        registry_address: RegistryContract::new(deps, abstract_code_id)?.address,
    })
}

pub(crate) fn ica_action(
    deps: Deps,
    env: Env,
    _account_address: String,
    chain: TruncatedChainId,
    actions: Vec<IcaAction>,
) -> IcaClientResult<IcaActionResponse> {
    // match chain-id with cosmos or EVM
    use abstract_ica::CastChainType;
    let chain_type = chain.chain_type().ok_or(IcaClientError::NoChainType {
        chain: chain.to_string(),
    })?;

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
                    let abstract_code_id = native_addrs::abstract_code_id(
                        &deps.querier,
                        env.contract.address.clone(),
                    )?;
                    let registry = RegistryContract::new(deps, abstract_code_id)?;

                    let msg = evm::execute(&deps.querier, &registry, msgs, callback)?;

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
                    deps, &env, &chain, funds, receiver, memo,
                )?]),
                _ => unimplemented!(),
            },
            _ => unimplemented!(),
        }
    };

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
        registry::{self as vc, ModuleConfiguration},
    };
    use abstract_testing::prelude::*;
    use cosmwasm_std::{
        from_json,
        testing::{mock_dependencies, MockApi},
        Addr, HexBinary,
    };

    use evm::types;
    use polytone_evm::EVM_NOTE_ID;

    type IbcClientTestResult = Result<(), IcaClientError>;

    const EVM_CHAIN: &str = "bartio";
    const COSMOS_CHAIN: &str = "juno";

    fn env_note_addr(api: MockApi) -> Addr {
        api.addr_make("evm_note_addr")
    }

    fn ucs_forwarder_addr(api: MockApi) -> Addr {
        api.addr_make("ucs_forwarder")
    }

    /// setup the querier with the proper responses and state
    fn state_setup(api: MockApi) -> MockQuerierBuilder {
        let chain_name = TruncatedChainId::from_str(EVM_CHAIN).unwrap();
        let abstr = AbstractMockAddrs::new(api);

        MockQuerierBuilder::new(api)
            .account(&abstr.account, TEST_ACCOUNT_ID)
            .contracts(vec![(
                &ContractEntry {
                    contract: types::UCS01_FORWARDER_CONTRACT.to_string(),
                    protocol: types::UCS01_PROTOCOL.to_string(),
                },
                ucs_forwarder_addr(api),
            )])
            .channels(vec![(
                &ChannelEntry {
                    connected_chain: chain_name.clone(),
                    protocol: types::UCS01_PROTOCOL.to_string(),
                },
                "channel-1".into(),
            )])
            .with_smart_handler(&env_note_addr(api), |bin| {
                let msg = from_json::<evm_note::msg::QueryMsg>(bin).unwrap();
                match msg {
                    evm_note::msg::QueryMsg::RemoteAddress { .. } => {
                        to_json_binary(&Some("123fff".to_owned())).map_err(|e| e.to_string())
                    }
                    _ => panic!("should only query for RemoteAddress"),
                }
            })
            .with_smart_handler(&abstr.registry, move |bin| {
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
                                    reference: ModuleReference::Native(env_note_addr(api)),
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
        use abstract_std::{ibc::PACKET_LIFETIME, objects::TruncatedChainId};

        use abstract_testing::mock_env_validated;
        use cosmwasm_std::{coins, wasm_execute};
        use evm::types;
        use evm_note::msg::EvmMsg;

        use types::Ucs01ForwarderExecuteMsg;

        #[coverage_helper::test]
        fn config() -> IbcClientTestResult {
            let mut deps = mock_dependencies();
            let env = mock_env_validated(deps.api);
            let abstr = AbstractMockAddrs::new(deps.api);

            deps.querier = state_setup(deps.api).build();

            mock_init(&mut deps)?;
            let res = query(deps.as_ref(), env, QueryMsg::Config {})?;
            let res: ConfigResponse = from_json(&res).unwrap();
            assert_eq!(
                res,
                ConfigResponse {
                    ans_host: abstr.ans_host,
                    registry_address: abstr.registry
                }
            );
            Ok(())
        }

        #[coverage_helper::test]
        fn evm_exec_no_callback() -> IbcClientTestResult {
            let mut deps = mock_dependencies();
            let env = mock_env_validated(deps.api);
            let abstr = AbstractMockAddrs::new(deps.api);
            let chain_name = TruncatedChainId::from_str(EVM_CHAIN)?;

            deps.querier = state_setup(deps.api).build();

            mock_init(&mut deps)?;

            let msg = QueryMsg::IcaAction {
                account_address: abstr.account.addr().to_string(),
                chain: chain_name,
                actions: vec![IcaAction::Execute(abstract_ica::IcaExecute::Evm {
                    msgs: vec![EvmMsg::Call {
                        target: "to".to_string(),
                        call_data: vec![0x01].into(),
                        value: None,
                        allow_failure: None,
                    }],
                    callback: None,
                })],
            };

            let res = query(deps.as_ref(), env, msg)?;
            let res: IcaActionResponse = from_json(&res).unwrap();

            assert_eq!(
                res,
                IcaActionResponse {
                    msgs: vec![CosmosMsg::Wasm(wasm_execute(
                        env_note_addr(deps.api),
                        &evm_note::msg::ExecuteMsg::Execute {
                            callback: None,
                            msgs: vec![EvmMsg::Call {
                                target: "to".to_string(),
                                call_data: vec![0x01].into(),
                                value: None,
                                allow_failure: None,
                            }],
                            timeout_seconds: PACKET_LIFETIME.into(),
                        },
                        vec![],
                    )?)],
                }
            );

            Ok(())
        }

        #[coverage_helper::test]
        fn evm_fund_no_callback() -> IbcClientTestResult {
            use super::*;

            let mut deps = mock_dependencies();
            let env = mock_env_validated(deps.api);
            let abstr = AbstractMockAddrs::new(deps.api);
            let chain_name = TruncatedChainId::from_str(EVM_CHAIN)?;

            deps.querier = state_setup(deps.api).build();

            mock_init(&mut deps)?;

            let receiver = HexBinary::from_hex("123fff").unwrap();

            let msg = QueryMsg::IcaAction {
                account_address: abstr.account.addr().to_string(),
                chain: chain_name,
                actions: vec![IcaAction::Fund {
                    funds: coins(1, "test"),
                    receiver: Some(receiver.clone().into()),
                    memo: None,
                }],
            };

            let res = query(deps.as_ref(), env, msg)?;
            let res: IcaActionResponse = from_json(&res).unwrap();

            assert_eq!(
                res,
                IcaActionResponse {
                    msgs: vec![CosmosMsg::Wasm(wasm_execute(
                        ucs_forwarder_addr(deps.api),
                        &Ucs01ForwarderExecuteMsg::Transfer {
                            channel: "channel-1".into(),
                            receiver,
                            memo: "".to_string(),
                            timeout: PACKET_LIFETIME.into(),
                        },
                        coins(1, "test"),
                    )?)],
                }
            );

            Ok(())
        }

        #[coverage_helper::test]
        fn evm_fund_no_receiver() -> IbcClientTestResult {
            use super::*;

            let mut deps = mock_dependencies();
            let abstr = AbstractMockAddrs::new(deps.api);
            let chain_name = TruncatedChainId::from_str(EVM_CHAIN)?;

            deps.querier = state_setup(deps.api).build();

            mock_init(&mut deps)?;

            let msg = QueryMsg::IcaAction {
                account_address: abstr.account.addr().to_string(),
                chain: chain_name,
                actions: vec![IcaAction::Fund {
                    funds: coins(1, "test"),
                    receiver: None,
                    memo: None,
                }],
            };

            let res = query(deps.as_ref(), mock_env_validated(deps.api), msg)?;
            let res: IcaActionResponse = from_json(&res).unwrap();

            assert_eq!(
                res,
                IcaActionResponse {
                    msgs: vec![CosmosMsg::Wasm(wasm_execute(
                        ucs_forwarder_addr(deps.api),
                        &Ucs01ForwarderExecuteMsg::Transfer {
                            channel: "channel-1".into(),
                            receiver: HexBinary::from_hex("123fff").unwrap(),
                            memo: "".to_string(),
                            timeout: PACKET_LIFETIME.into(),
                        },
                        coins(1, "test"),
                    )?)],
                }
            );

            Ok(())
        }

        #[coverage_helper::test]
        fn evm_exec_non_evm_chaintype() -> IbcClientTestResult {
            let mut deps = mock_dependencies();
            let abstr = AbstractMockAddrs::new(deps.api);
            let chain_name = TruncatedChainId::from_str(COSMOS_CHAIN)?;

            deps.querier = state_setup(deps.api).build();

            mock_init(&mut deps)?;

            let msg = QueryMsg::IcaAction {
                account_address: abstr.account.addr().to_string(),
                chain: chain_name.clone(),
                actions: vec![IcaAction::Execute(abstract_ica::IcaExecute::Evm {
                    msgs: vec![EvmMsg::Call {
                        target: "to".to_string(),
                        call_data: vec![0x01].into(),
                        value: None,
                        allow_failure: None,
                    }],
                    callback: None,
                })],
            };

            let err = query(deps.as_ref(), mock_env_validated(deps.api), msg).unwrap_err();
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
