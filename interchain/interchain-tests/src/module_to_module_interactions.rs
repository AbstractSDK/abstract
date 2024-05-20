pub use abstract_std::app;
use abstract_std::{
    ibc::{CallbackInfo, CallbackResult, ModuleIbcMsg},
    ibc_client,
    objects::{chain_name::ChainName, module::ModuleInfo},
    IBC_CLIENT,
};
use cosmwasm_schema::{cw_serde, QueryResponses};
pub use cosmwasm_std::testing::{mock_dependencies, mock_env, mock_info};
use cosmwasm_std::{
    from_json, to_json_binary, wasm_execute, AllBalanceResponse, Coin, Response, StdError,
};
use cw_controllers::AdminError;
use cw_storage_plus::Item;

pub type AppTestResult = Result<(), MockError>;

abstract_app::app_msg_types!(MockAppContract, MockExecMsg, MockQueryMsg);

#[cosmwasm_schema::cw_serde]
pub struct MockInitMsg {}

#[cosmwasm_schema::cw_serde]
#[derive(cw_orch::ExecuteFns)]
#[impl_into(ExecuteMsg)]
pub enum MockExecMsg {
    DoSomething {},
    DoSomethingAdmin {},
    DoSomethingIbc {
        remote_chain: ChainName,
        target_module: ModuleInfo,
    },
    QuerySomethingIbc {
        remote_chain: ChainName,
        address: String,
    },
}

#[cosmwasm_schema::cw_serde]
#[derive(cw_orch::QueryFns)]
#[impl_into(QueryMsg)]
#[derive(QueryResponses)]
pub enum MockQueryMsg {
    #[returns(ReceivedIbcCallbackStatus)]
    GetReceivedIbcCallbackStatus {},

    #[returns(ReceivedIbcQueryCallbackStatus)]
    GetReceivedIbcQueryCallbackStatus {},

    #[returns(ReceivedIbcModuleStatus)]
    GetReceivedIbcModuleStatus {},
}

#[cosmwasm_schema::cw_serde]
pub struct ReceivedIbcCallbackStatus {
    pub received: bool,
}

#[cosmwasm_schema::cw_serde]
pub struct ReceivedIbcQueryCallbackStatus {
    pub balance: Vec<Coin>,
}

#[cosmwasm_schema::cw_serde]
pub struct ReceivedIbcModuleStatus {
    pub received: ModuleInfo,
}

#[cosmwasm_schema::cw_serde]
pub struct MockMigrateMsg;

#[cosmwasm_schema::cw_serde]
pub struct MockReceiveMsg;

#[cosmwasm_schema::cw_serde]
pub struct MockSudoMsg;

use abstract_sdk::{AbstractSdkError, ModuleInterface};
use thiserror::Error;

use abstract_app::{AppContract, AppError};

#[derive(Error, Debug, PartialEq)]
pub enum MockError {
    #[error("{0}")]
    Std(#[from] StdError),

    #[error("{0}")]
    DappError(#[from] AppError),

    #[error("{0}")]
    Abstract(#[from] abstract_std::AbstractError),

    #[error("{0}")]
    AbstractSdk(#[from] AbstractSdkError),

    #[error("{0}")]
    Admin(#[from] AdminError),
}

pub type MockAppContract = AppContract<
    // MockModule,
    MockError,
    MockInitMsg,
    MockExecMsg,
    MockQueryMsg,
    MockMigrateMsg,
    MockReceiveMsg,
    MockSudoMsg,
>;

#[cw_serde]
pub struct IbcModuleToModuleMsg {
    ibc_msg: String,
}

// Easy way to see if an ibc-callback was actually received.
pub const IBC_CALLBACK_RECEIVED: Item<bool> = Item::new("ibc_callback_received");
// Easy way to see if an module ibc called was actually received.
pub const MODULE_IBC_RECEIVED: Item<ModuleInfo> = Item::new("module_ibc_received");

// Easy way to see if an ibc-callback was actually received.
pub const IBC_CALLBACK_QUERY_RECEIVED: Item<Vec<Coin>> = Item::new("ibc_callback_query_received");

pub const fn mock_app(id: &'static str, version: &'static str) -> MockAppContract {
    MockAppContract::new(id, version, None)
        .with_instantiate(|deps, _, _, _, _| {
            IBC_CALLBACK_RECEIVED.save(deps.storage, &false)?;
            Ok(Response::new().set_data("mock_init".as_bytes()))
        })
        .with_execute(|deps, _env, _, app, msg| match msg {
            MockExecMsg::DoSomethingIbc {
                remote_chain,
                target_module,
            } => {
                let ibc_client_addr = app.modules(deps.as_ref()).module_address(IBC_CLIENT)?;
                // We send an IBC Client module message
                let msg = wasm_execute(
                    ibc_client_addr,
                    &ibc_client::ExecuteMsg::ModuleIbcAction {
                        host_chain: remote_chain,
                        target_module,
                        msg: to_json_binary(&IbcModuleToModuleMsg {
                            ibc_msg: "module_to_module:msg".to_string(),
                        })
                        .unwrap(),
                        callback_info: Some(CallbackInfo {
                            id: "c_id".to_string(),
                            msg: None,
                        }),
                    },
                    vec![],
                )?;

                Ok(Response::new().add_message(msg))
            }
            MockExecMsg::QuerySomethingIbc {
                address,
                remote_chain,
            } => {
                let ibc_client_addr = app.modules(deps.as_ref()).module_address(IBC_CLIENT)?;
                // We send an IBC Client module message
                let msg = wasm_execute(
                    ibc_client_addr,
                    &ibc_client::ExecuteMsg::IbcQuery {
                        host_chain: remote_chain,
                        callback_info: CallbackInfo {
                            id: "query_id".to_string(),
                            msg: None,
                        },
                        query: cosmwasm_std::QueryRequest::Bank(
                            cosmwasm_std::BankQuery::AllBalances { address },
                        ),
                    },
                    vec![],
                )?;

                Ok(Response::new().add_message(msg))
            }
            _ => Ok(Response::new().set_data("mock_exec".as_bytes())),
        })
        .with_query(|deps, _, _, msg| match msg {
            MockQueryMsg::GetReceivedIbcCallbackStatus {} => {
                to_json_binary(&ReceivedIbcCallbackStatus {
                    received: IBC_CALLBACK_RECEIVED.load(deps.storage)?,
                })
                .map_err(Into::into)
            }
            MockQueryMsg::GetReceivedIbcModuleStatus {} => {
                to_json_binary(&ReceivedIbcModuleStatus {
                    received: MODULE_IBC_RECEIVED.load(deps.storage)?,
                })
                .map_err(Into::into)
            }
            MockQueryMsg::GetReceivedIbcQueryCallbackStatus {} => {
                to_json_binary(&ReceivedIbcQueryCallbackStatus {
                    balance: IBC_CALLBACK_QUERY_RECEIVED.load(deps.storage)?,
                })
                .map_err(Into::into)
            }
        })
        .with_sudo(|_, _, _, _| Ok(Response::new().set_data("mock_sudo".as_bytes())))
        .with_receive(|_, _, _, _, _| Ok(Response::new().set_data("mock_receive".as_bytes())))
        .with_ibc_callbacks(&[
            ("c_id", |deps, _, _, _, _| {
                IBC_CALLBACK_RECEIVED.save(deps.storage, &true).unwrap();
                Ok(Response::new().add_attribute("mock_callback", "executed"))
            }),
            ("query_id", |deps, _, _, _, msg| match msg.result {
                CallbackResult::Query { query: _, result } => {
                    let result = result.unwrap()[0].clone();
                    let deser: AllBalanceResponse = from_json(result)?;
                    IBC_CALLBACK_QUERY_RECEIVED
                        .save(deps.storage, &deser.amount)
                        .unwrap();
                    Ok(Response::new().add_attribute("mock_callback_query", "executed"))
                }
                _ => panic!("Expected query result"),
            }),
        ])
        .with_replies(&[(1u64, |_, _, _, msg| {
            Ok(Response::new().set_data(msg.result.unwrap().data.unwrap()))
        })])
        .with_migrate(|_, _, _, _| Ok(Response::new().set_data("mock_migrate".as_bytes())))
        .with_module_ibc(|deps, _, _, msg| {
            let ModuleIbcMsg { source_module, .. } = msg;
            // We save the module info status
            MODULE_IBC_RECEIVED.save(deps.storage, &source_module)?;
            Ok(Response::new().add_attribute("mock_module_ibc", "executed"))
        })
}

pub mod origin_app {
    use abstract_testing::addresses::{TEST_MODULE_ID, TEST_VERSION};

    use super::{mock_app, MockAppContract};
    pub const MOCK_APP_ORIGIN: MockAppContract = mock_app(TEST_MODULE_ID, TEST_VERSION);
    abstract_app::cw_orch_interface!(MOCK_APP_ORIGIN, MockAppContract, MockAppOriginI);
}

pub mod remote_app {
    use super::{mock_app, MockAppContract};

    pub const TEST_MODULE_ID_REMOTE: &str = "tester:test-module-id-remote";
    pub const TEST_VERSION_REMOTE: &str = "0.45.7";
    pub const MOCK_APP_REMOTE: MockAppContract =
        mock_app(TEST_MODULE_ID_REMOTE, TEST_VERSION_REMOTE);
    abstract_app::cw_orch_interface!(MOCK_APP_REMOTE, MockAppContract, MockAppRemoteI);
}

#[cfg(test)]
pub mod test {

    fn assert_remote_module_call_status(
        app: &MockAppRemoteI<MockBech32>,
        source_module_expected: Option<ModuleInfo>,
    ) -> AnyResult<()> {
        let source_module = app
            .get_received_ibc_module_status()
            .map(|s| s.received)
            .ok();

        assert_eq!(source_module, source_module_expected);
        Ok(())
    }

    fn assert_callback_status(app: &MockAppOriginI<MockBech32>, status: bool) -> AnyResult<()> {
        let get_received_ibc_callback_status_res: ReceivedIbcCallbackStatus =
            app.get_received_ibc_callback_status()?;

        assert_eq!(
            ReceivedIbcCallbackStatus { received: status },
            get_received_ibc_callback_status_res
        );
        Ok(())
    }

    fn assert_query_callback_status(
        app: &MockAppOriginI<MockBech32>,
        balance: Vec<Coin>,
    ) -> AnyResult<()> {
        let get_received_ibc_query_callback_status_res: ReceivedIbcQueryCallbackStatus =
            app.get_received_ibc_query_callback_status()?;

        assert_eq!(
            ReceivedIbcQueryCallbackStatus { balance },
            get_received_ibc_query_callback_status_res
        );
        Ok(())
    }
    use crate::{
        interchain_accounts::create_test_remote_account,
        module_to_module_interactions::{
            origin_app::interface::MockAppOriginI,
            remote_app::{interface::MockAppRemoteI, TEST_MODULE_ID_REMOTE, TEST_VERSION_REMOTE},
            MockExecMsgFns, MockInitMsg, MockQueryMsgFns, ReceivedIbcCallbackStatus,
            ReceivedIbcQueryCallbackStatus,
        },
        setup::{
            ibc_abstract_setup, ibc_connect_polytone_and_abstract, mock_test::logger_test_init,
        },
        JUNO, STARGAZE,
    };
    use abstract_app::objects::{chain_name::ChainName, module::ModuleInfo};
    use abstract_interface::{
        AppDeployer, DeployStrategy, Manager, ManagerQueryFns, VCExecFns, VCQueryFns,
    };
    use abstract_std::manager::{self, ModuleInstallConfig};
    use abstract_testing::addresses::{TEST_MODULE_ID, TEST_NAMESPACE, TEST_VERSION};
    use anyhow::Result as AnyResult;
    use base64::{engine::general_purpose, Engine};
    use cosmwasm_std::{coins, to_json_binary};
    use cw_orch::interchain::MockBech32InterchainEnv;
    use cw_orch::prelude::*;

    #[test]
    fn target_module_must_exist() -> AnyResult<()> {
        logger_test_init();
        let mock_interchain =
            MockBech32InterchainEnv::new(vec![(JUNO, "juno"), (STARGAZE, "stargaze")]);

        // We just verified all steps pass
        let (abstr_origin, _abstr_remote) = ibc_abstract_setup(&mock_interchain, JUNO, STARGAZE)?;
        ibc_connect_polytone_and_abstract(&mock_interchain, STARGAZE, JUNO)?;

        let remote_name = ChainName::from_chain_id(STARGAZE);

        let (origin_account, _remote_account_id) =
            create_test_remote_account(&abstr_origin, JUNO, STARGAZE, &mock_interchain, None)?;

        let app = MockAppOriginI::new(
            TEST_MODULE_ID,
            abstr_origin.version_control.get_chain().clone(),
        );

        abstr_origin.version_control.claim_namespace(
            origin_account.manager.config()?.account_id,
            TEST_NAMESPACE.to_owned(),
        )?;

        app.deploy(TEST_VERSION.parse()?, DeployStrategy::Try)?;

        origin_account.install_app(&app, &MockInitMsg {}, None)?;

        // The user on origin chain wants to change the account description
        let target_module_info =
            ModuleInfo::from_id(TEST_MODULE_ID_REMOTE, TEST_VERSION_REMOTE.into())?;
        let ibc_action_result = app.do_something_ibc(remote_name, target_module_info.clone())?;

        let ibc_result = mock_interchain.wait_ibc(JUNO, ibc_action_result)?;

        let expected_error_outcome = format!(
            "Module {} does not have a stored module reference",
            target_module_info
        );
        assert!(ibc_result
            .into_result()
            .unwrap_err()
            .to_string()
            .contains(&expected_error_outcome));
        // match &ibc_result.packets[0].outcome {
        //     cw_orch::interchain::types::IbcPacketOutcome::Timeout { .. } => {
        //         panic!("Expected a failed ack not a timeout !")
        //     }
        //     cw_orch::interchain::types::IbcPacketOutcome::Success { ack, .. } => match ack {
        //         cw_orch::interchain::types::IbcPacketAckDecode::Error(e) => {
        //             assert!(e.contains(&expected_error_outcome));
        //         }
        //         cw_orch::interchain::types::IbcPacketAckDecode::Success(_) => {
        //             panic!("Expected a error ack")
        //         }
        //         cw_orch::interchain::types::IbcPacketAckDecode::NotParsed(original_ack) => {
        //             let error_str =
        //                 String::from_utf8_lossy(&general_purpose::STANDARD.decode(original_ack)?)
        //                     .to_string();
        //             assert!(error_str.contains(&expected_error_outcome));
        //         }
        //     },
        // }

        Ok(())
    }

    #[test]
    fn target_account_must_have_module_installed() -> AnyResult<()> {
        logger_test_init();
        let mock_interchain =
            MockBech32InterchainEnv::new(vec![(JUNO, "juno"), (STARGAZE, "stargaze")]);

        // We just verified all steps pass
        let (abstr_origin, abstr_remote) = ibc_abstract_setup(&mock_interchain, JUNO, STARGAZE)?;
        ibc_connect_polytone_and_abstract(&mock_interchain, STARGAZE, JUNO)?;

        let remote_name = ChainName::from_chain_id(STARGAZE);

        let (origin_account, _remote_account_id) =
            create_test_remote_account(&abstr_origin, JUNO, STARGAZE, &mock_interchain, None)?;

        let (remote_account, _remote_account_id) =
            create_test_remote_account(&abstr_remote, STARGAZE, JUNO, &mock_interchain, None)?;

        // Install local app
        let app = MockAppOriginI::new(
            TEST_MODULE_ID,
            abstr_origin.version_control.get_chain().clone(),
        );

        abstr_origin
            .version_control
            .claim_namespace(origin_account.id()?, TEST_NAMESPACE.to_owned())?;

        app.deploy(TEST_VERSION.parse()?, DeployStrategy::Try)?;

        origin_account.install_app(&app, &MockInitMsg {}, None)?;

        // Install remote app
        let app_remote = MockAppRemoteI::new(
            TEST_MODULE_ID_REMOTE,
            abstr_remote.version_control.get_chain().clone(),
        );

        abstr_remote
            .version_control
            .claim_namespace(remote_account.id()?, TEST_NAMESPACE.to_owned())?;

        app_remote.deploy(TEST_VERSION_REMOTE.parse()?, DeployStrategy::Try)?;

        // The user on origin chain wants to change the account description
        let target_module_info =
            ModuleInfo::from_id(TEST_MODULE_ID_REMOTE, TEST_VERSION_REMOTE.into())?;
        let ibc_action_result = app.do_something_ibc(remote_name, target_module_info.clone())?;

        let ibc_result = mock_interchain.wait_ibc(JUNO, ibc_action_result)?;

        let expected_error_outcome =
            format!("App {} not installed on Account", target_module_info,);

        assert!(ibc_result
            .into_result()
            .unwrap_err()
            .to_string()
            .contains(&expected_error_outcome));
        // match &ibc_result.packets[0].outcome {
        //     cw_orch::interchain::types::IbcPacketOutcome::Timeout { .. } => {
        //         panic!("Expected a failed ack not a timeout !")
        //     }
        //     cw_orch::interchain::types::IbcPacketOutcome::Success { ack, .. } => match ack {
        //         cw_orch::interchain::types::IbcPacketAckDecode::Error(e) => {
        //             assert!(e.contains(&expected_error_outcome));
        //         }
        //         cw_orch::interchain::types::IbcPacketAckDecode::Success(_) => {
        //             panic!("Expected a error ack")
        //         }
        //         cw_orch::interchain::types::IbcPacketAckDecode::NotParsed(original_ack) => {
        //             let error_str =
        //                 String::from_utf8_lossy(&general_purpose::STANDARD.decode(original_ack)?)
        //                     .to_string();
        //             assert!(error_str.contains(&expected_error_outcome));
        //         }
        //     },
        // }

        Ok(())
    }

    #[test]
    fn works() -> AnyResult<()> {
        logger_test_init();
        let mock_interchain =
            MockBech32InterchainEnv::new(vec![(JUNO, "juno"), (STARGAZE, "stargaze")]);

        // We just verified all steps pass
        let (abstr_origin, abstr_remote) = ibc_abstract_setup(&mock_interchain, JUNO, STARGAZE)?;
        ibc_connect_polytone_and_abstract(&mock_interchain, STARGAZE, JUNO)?;

        let remote_name = ChainName::from_chain_id(STARGAZE);

        let (origin_account, remote_account_id) =
            create_test_remote_account(&abstr_origin, JUNO, STARGAZE, &mock_interchain, None)?;

        let (remote_account, _) =
            create_test_remote_account(&abstr_remote, STARGAZE, JUNO, &mock_interchain, None)?;

        // Install local app
        let app = MockAppOriginI::new(
            TEST_MODULE_ID,
            abstr_origin.version_control.get_chain().clone(),
        );

        abstr_origin
            .version_control
            .claim_namespace(origin_account.id()?, TEST_NAMESPACE.to_owned())?;

        app.deploy(TEST_VERSION.parse()?, DeployStrategy::Try)?;

        origin_account.install_app(&app, &MockInitMsg {}, None)?;

        // Install remote app
        let app_remote = MockAppRemoteI::new(
            TEST_MODULE_ID_REMOTE,
            abstr_remote.version_control.get_chain().clone(),
        );

        abstr_remote
            .version_control
            .claim_namespace(remote_account.id()?, TEST_NAMESPACE.to_owned())?;

        app_remote.deploy(TEST_VERSION_REMOTE.parse()?, DeployStrategy::Try)?;

        let remote_install_response = origin_account.manager.execute_on_remote(
            remote_name.clone(),
            manager::ExecuteMsg::InstallModules {
                modules: vec![ModuleInstallConfig::new(
                    ModuleInfo::from_id_latest(TEST_MODULE_ID_REMOTE)?,
                    Some(to_json_binary(&MockInitMsg {})?),
                )],
            },
        )?;

        mock_interchain.wait_ibc(JUNO, remote_install_response)?;

        // We get the object for handling the actual module on the remote account
        let remote_manager = abstr_remote
            .version_control
            .account_base(remote_account_id)?
            .account_base
            .manager;
        let manager = Manager::new(
            "remote-account-manager",
            abstr_remote.version_control.get_chain().clone(),
        );
        manager.set_address(&remote_manager);
        let module_address = manager.module_info(TEST_MODULE_ID_REMOTE)?.unwrap().address;
        let remote_account_app = MockAppRemoteI::new(
            "remote-account-app",
            abstr_remote.version_control.get_chain().clone(),
        );
        remote_account_app.set_address(&module_address);

        // The user on origin chain triggers a module-to-module interaction
        let target_module_info =
            ModuleInfo::from_id(TEST_MODULE_ID_REMOTE, TEST_VERSION_REMOTE.into())?;
        let ibc_action_result = app.do_something_ibc(remote_name, target_module_info.clone())?;

        assert_remote_module_call_status(&remote_account_app, None)?;
        assert_callback_status(&app, false)?;

        mock_interchain.wait_ibc(JUNO, ibc_action_result)?;

        assert_remote_module_call_status(
            &remote_account_app,
            Some(ModuleInfo::from_id(TEST_MODULE_ID, TEST_VERSION.into())?),
        )?;
        assert_callback_status(&app, true)?;

        Ok(())
    }

    pub const REMOTE_AMOUNT: u128 = 5674309;
    pub const REMOTE_DENOM: &str = "remote_denom";
    #[test]
    fn queries() -> AnyResult<()> {
        logger_test_init();
        let mock_interchain =
            MockBech32InterchainEnv::new(vec![(JUNO, "juno"), (STARGAZE, "stargaze")]);

        // We just verified all steps pass
        let (abstr_origin, _abstr_remote) = ibc_abstract_setup(&mock_interchain, JUNO, STARGAZE)?;
        ibc_connect_polytone_and_abstract(&mock_interchain, STARGAZE, JUNO)?;

        let remote_name = ChainName::from_chain_id(STARGAZE);
        let remote = mock_interchain.chain(STARGAZE)?;
        let remote_address =
            remote.addr_make_with_balance("remote-test", coins(REMOTE_AMOUNT, REMOTE_DENOM))?;

        let (origin_account, _remote_account_id) =
            create_test_remote_account(&abstr_origin, JUNO, STARGAZE, &mock_interchain, None)?;

        // Install local app
        let app = MockAppOriginI::new(
            TEST_MODULE_ID,
            abstr_origin.version_control.get_chain().clone(),
        );

        abstr_origin
            .version_control
            .claim_namespace(origin_account.id()?, TEST_NAMESPACE.to_owned())?;

        app.deploy(TEST_VERSION.parse()?, DeployStrategy::Try)?;

        origin_account.install_app(&app, &MockInitMsg {}, None)?;

        let query_response = app.query_something_ibc(remote_address.to_string(), remote_name)?;

        assert_query_callback_status(&app, coins(REMOTE_AMOUNT, REMOTE_DENOM)).unwrap_err();
        mock_interchain.wait_ibc(JUNO, query_response)?;
        assert_query_callback_status(&app, coins(REMOTE_AMOUNT, REMOTE_DENOM))?;

        Ok(())
    }

    pub mod security {
        use abstract_std::ibc_client::ExecuteMsgFns;

        use crate::module_to_module_interactions::IbcModuleToModuleMsg;

        use super::*;

        #[test]
        fn calling_module_should_match() -> AnyResult<()> {
            logger_test_init();
            let mock_interchain =
                MockBech32InterchainEnv::new(vec![(JUNO, "juno"), (STARGAZE, "stargaze")]);

            // We just verified all steps pass
            let (abstr_origin, abstr_remote) =
                ibc_abstract_setup(&mock_interchain, JUNO, STARGAZE)?;
            ibc_connect_polytone_and_abstract(&mock_interchain, STARGAZE, JUNO)?;

            let remote_name = ChainName::from_chain_id(STARGAZE);

            let (origin_account, remote_account_id) =
                create_test_remote_account(&abstr_origin, JUNO, STARGAZE, &mock_interchain, None)?;

            let (remote_account, _) =
                create_test_remote_account(&abstr_remote, STARGAZE, JUNO, &mock_interchain, None)?;

            // Install local app
            let app = MockAppOriginI::new(
                TEST_MODULE_ID,
                abstr_origin.version_control.get_chain().clone(),
            );

            abstr_origin
                .version_control
                .claim_namespace(origin_account.id()?, TEST_NAMESPACE.to_owned())?;

            app.deploy(TEST_VERSION.parse()?, DeployStrategy::Try)?;

            origin_account.install_app(&app, &MockInitMsg {}, None)?;

            // Install remote app
            let app_remote = MockAppRemoteI::new(
                TEST_MODULE_ID_REMOTE,
                abstr_remote.version_control.get_chain().clone(),
            );

            abstr_remote
                .version_control
                .claim_namespace(remote_account.id()?, TEST_NAMESPACE.to_owned())?;

            app_remote.deploy(TEST_VERSION_REMOTE.parse()?, DeployStrategy::Try)?;

            let remote_install_response = origin_account.manager.execute_on_remote(
                remote_name.clone(),
                manager::ExecuteMsg::InstallModules {
                    modules: vec![ModuleInstallConfig::new(
                        ModuleInfo::from_id_latest(TEST_MODULE_ID_REMOTE)?,
                        Some(to_json_binary(&MockInitMsg {})?),
                    )],
                },
            )?;

            mock_interchain.wait_ibc(JUNO, remote_install_response)?;

            // We get the object for handling the actual module on the remote account
            let remote_manager = abstr_remote
                .version_control
                .account_base(remote_account_id)?
                .account_base
                .manager;
            let manager = Manager::new(
                "remote-account-manager",
                abstr_remote.version_control.get_chain().clone(),
            );
            manager.set_address(&remote_manager);
            let module_address = manager.module_info(TEST_MODULE_ID_REMOTE)?.unwrap().address;
            let remote_account_app = MockAppRemoteI::new(
                "remote-account-app",
                abstr_remote.version_control.get_chain().clone(),
            );
            remote_account_app.set_address(&module_address);

            // The user on origin chain triggers a module-to-module interaction
            let target_module_info =
                ModuleInfo::from_id(TEST_MODULE_ID_REMOTE, TEST_VERSION_REMOTE.into())?;

            // The user triggers manually a module-to-module interaction
            abstr_origin
                .ibc
                .client
                .module_ibc_action(
                    remote_name,
                    to_json_binary(&IbcModuleToModuleMsg {
                        ibc_msg: "module_to_module:msg".to_string(),
                    })
                    .unwrap(),
                    target_module_info,
                    None,
                )
                .unwrap_err();

            Ok(())
        }
    }
}
