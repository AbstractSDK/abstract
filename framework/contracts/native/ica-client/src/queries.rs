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
    actions: Vec<IcaAction>,
) -> IcaClientResult<IcaActionResponse> {
    // match chain-id with cosmos or EVM
    use abstract_ica::CastChainType;
    let chain_type = chain.chain_type().ok_or(IcaClientError::NoChainType {
        chain: chain.to_string(),
    })?;

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
    use abstract_testing::prelude::*;
    use cosmwasm_std::{
        from_json,
        testing::{mock_dependencies, mock_env, MockApi},
        Addr, HexBinary,
    };

    use evm::types;
    use polytone_evm::EVM_NOTE_ID;
    use speculoos::prelude::*;

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

        AbstractMockQuerierBuilder::new(api)
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
            .builder()
            .with_smart_handler(&env_note_addr(api), |bin| {
                let msg = from_json::<evm_note::msg::QueryMsg>(bin).unwrap();
                match msg {
                    evm_note::msg::QueryMsg::RemoteAddress { .. } => {
                        to_json_binary(&Some("123fff".to_owned())).map_err(|e| e.to_string())
                    }
                    _ => panic!("should only query for RemoteAddress"),
                }
            })
            .with_smart_handler(&abstr.version_control, move |bin| {
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
        use abstract_std::objects::TruncatedChainId;

        use cosmwasm_std::{coins, wasm_execute};
        use evm::types;
        use evm_note::msg::EvmMsg;

        use types::Ucs01ForwarderExecuteMsg;

        #[test]
        fn config() -> IbcClientTestResult {
            let mut deps = mock_dependencies();
            let abstr = AbstractMockAddrs::new(deps.api);

            deps.querier = state_setup(deps.api).build();

            mock_init(&mut deps)?;
            let res = query(deps.as_ref(), mock_env(), QueryMsg::Config {})?;
            let res: ConfigResponse = from_json(&res).unwrap();
            assert_eq!(
                res,
                ConfigResponse {
                    ans_host: abstr.ans_host.to_string(),
                    version_control_address: abstr.version_control.to_string()
                }
            );
            Ok(())
        }

        #[test]
        fn evm_exec_no_callback() -> IbcClientTestResult {
            let mut deps = mock_dependencies();
            let abstr = AbstractMockAddrs::new(deps.api);
            let chain_name = TruncatedChainId::from_str(EVM_CHAIN)?;

            deps.querier = state_setup(deps.api).build();

            mock_init(&mut deps)?;

            let msg = QueryMsg::IcaAction {
                proxy_address: abstr.account.proxy.to_string(),
                chain: chain_name,
                actions: vec![IcaAction::Execute(abstract_ica::IcaExecute::Evm {
                    msgs: vec![EvmMsg::Call {
                        to: "to".to_string(),
                        data: vec![0x01].into(),
                    }],
                    callback: None,
                })],
            };

            let res = query(deps.as_ref(), mock_env(), msg)?;
            let res: IcaActionResponse = from_json(&res).unwrap();

            assert_that!(res).is_equal_to(IcaActionResponse {
                msgs: vec![CosmosMsg::Wasm(wasm_execute(
                    env_note_addr(deps.api),
                    &evm_note::msg::ExecuteMsg::Execute {
                        callback: None,
                        msgs: vec![EvmMsg::Call {
                            to: "to".to_string(),
                            data: vec![0x01].into(),
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
            let abstr = AbstractMockAddrs::new(deps.api);
            let chain_name = TruncatedChainId::from_str(EVM_CHAIN)?;

            deps.querier = state_setup(deps.api).build();

            mock_init(&mut deps)?;

            let receiver = HexBinary::from_hex("123fff").unwrap();

            let msg = QueryMsg::IcaAction {
                proxy_address: abstr.account.proxy.to_string(),
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
                    ucs_forwarder_addr(deps.api),
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
            let abstr = AbstractMockAddrs::new(deps.api);
            let chain_name = TruncatedChainId::from_str(EVM_CHAIN)?;

            deps.querier = state_setup(deps.api).build();

            mock_init(&mut deps)?;

            let msg = QueryMsg::IcaAction {
                proxy_address: abstr.account.proxy.to_string(),
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
                    ucs_forwarder_addr(deps.api),
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
            let abstr = AbstractMockAddrs::new(deps.api);
            let chain_name = TruncatedChainId::from_str(COSMOS_CHAIN)?;

            deps.querier = state_setup(deps.api).build();

            mock_init(&mut deps)?;

            let msg = QueryMsg::IcaAction {
                proxy_address: abstr.account.proxy.to_string(),
                chain: chain_name.clone(),
                actions: vec![IcaAction::Execute(abstract_ica::IcaExecute::Evm {
                    msgs: vec![EvmMsg::Call {
                        to: "to".to_string(),
                        data: vec![0x01].into(),
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
