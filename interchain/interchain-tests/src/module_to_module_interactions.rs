pub use abstract_std::app;
use abstract_std::{
    ibc::{Callback, IbcResult},
    ibc_client::{self, InstalledModuleIdentification},
    objects::{dependency::StaticDependency, module::ModuleInfo, TruncatedChainId},
    IBC_CLIENT,
};
use cosmwasm_schema::{cw_serde, QueryResponses};
pub use cosmwasm_std::testing::{mock_dependencies, mock_env};
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
pub enum MockExecMsg {
    DoSomething {},
    DoSomethingAdmin {},
    DoSomethingIbc {
        remote_chain: TruncatedChainId,
        target_module: ModuleInfo,
    },
    QuerySomethingIbc {
        remote_chain: TruncatedChainId,
        address: String,
    },
    QueryModuleIbc {
        remote_chain: TruncatedChainId,
        target_module: ModuleInfo,
    },
}

#[cosmwasm_schema::cw_serde]
#[derive(cw_orch::QueryFns, QueryResponses)]
pub enum MockQueryMsg {
    #[returns(ReceivedIbcCallbackStatus)]
    GetReceivedIbcCallbackStatus {},

    #[returns(ReceivedIbcQueryCallbackStatus)]
    GetReceivedIbcQueryCallbackStatus {},

    #[returns(ReceivedIbcModuleStatus)]
    GetReceivedIbcModuleStatus {},

    #[returns(String)]
    Foo {},

    #[returns(String)]
    GetReceivedModuleIbcQueryCallbackStatus {},
}

#[cw_serde]
pub enum MockCallbackMsg {
    BalanceQuery,
    ModuleQuery,
    ModuleExecute,
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
    pub received: Option<ModuleInfo>,
}

#[cosmwasm_schema::cw_serde]
pub struct MockMigrateMsg;

#[cosmwasm_schema::cw_serde]
pub struct MockReceiveMsg;

#[cosmwasm_schema::cw_serde]
pub struct MockSudoMsg;

use abstract_sdk::{AbstractSdkError, IbcInterface, ModuleInterface};
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

// Easy way to see if an ibc-query was actually performed.
pub const IBC_CALLBACK_MODULE_QUERY_RECEIVED: Item<String> =
    Item::new("ibc_callback_module_query_received");

pub const fn mock_app(id: &'static str, version: &'static str) -> MockAppContract {
    const IBC_CLIENT_DEP: StaticDependency =
        StaticDependency::new(IBC_CLIENT, &[abstract_std::registry::ABSTRACT_VERSION]);

    MockAppContract::new(id, version, None)
        .with_instantiate(|deps, _, _, _, _| {
            IBC_CALLBACK_RECEIVED.save(deps.storage, &false)?;
            IBC_CALLBACK_QUERY_RECEIVED.save(deps.storage, &vec![])?;
            IBC_CALLBACK_MODULE_QUERY_RECEIVED.save(deps.storage, &String::new())?;
            Ok(Response::new().set_data("mock_init".as_bytes()))
        })
        .with_execute(|deps, env, _, app, msg| match msg {
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
                        callback: Some(Callback {
                            msg: to_json_binary(&MockCallbackMsg::ModuleExecute)?,
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
                        callback: Callback {
                            msg: to_json_binary(&MockCallbackMsg::BalanceQuery)?,
                        },
                        queries: vec![cosmwasm_std::QueryRequest::Bank(
                            cosmwasm_std::BankQuery::AllBalances { address },
                        )],
                    },
                    vec![],
                )?;

                Ok(Response::new().add_message(msg))
            }
            MockExecMsg::QueryModuleIbc {
                remote_chain,
                target_module,
            } => {
                use abstract_sdk::features::AccountIdentification;
                let ibc_client = app.ibc_client(deps.as_ref());
                let mut account = app.account_id(deps.as_ref())?;
                account.push_chain(TruncatedChainId::new(&env));
                let msg = ibc_client.module_ibc_query(
                    remote_chain,
                    InstalledModuleIdentification {
                        module_info: target_module,
                        account_id: Some(account),
                    },
                    &QueryMsg::from(MockQueryMsg::Foo {}),
                    Callback {
                        msg: to_json_binary(&MockCallbackMsg::ModuleQuery)?,
                    },
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
                    received: MODULE_IBC_RECEIVED.may_load(deps.storage)?,
                })
                .map_err(Into::into)
            }
            MockQueryMsg::GetReceivedIbcQueryCallbackStatus {} => {
                to_json_binary(&ReceivedIbcQueryCallbackStatus {
                    balance: IBC_CALLBACK_QUERY_RECEIVED.load(deps.storage)?,
                })
                .map_err(Into::into)
            }
            MockQueryMsg::Foo {} => to_json_binary("bar").map_err(Into::into),
            MockQueryMsg::GetReceivedModuleIbcQueryCallbackStatus {} => Ok(to_json_binary(
                &IBC_CALLBACK_MODULE_QUERY_RECEIVED.load(deps.storage)?,
            )
            .unwrap()),
        })
        .with_sudo(|_, _, _, _| Ok(Response::new().set_data("mock_sudo".as_bytes())))
        .with_ibc_callback(|deps, _, _, callback, result| {
            eprintln!("{:?}", result);
            match &result {
                IbcResult::Query {
                    queries: _,
                    results,
                } => {
                    match from_json(callback.msg)? {
                        MockCallbackMsg::BalanceQuery => {
                            let result = results.clone().unwrap()[0].clone();
                            let deser: AllBalanceResponse = from_json(result)?;
                            IBC_CALLBACK_QUERY_RECEIVED
                                .save(deps.storage, &deser.amount)
                                .unwrap();
                        }
                        MockCallbackMsg::ModuleQuery => {
                            IBC_CALLBACK_MODULE_QUERY_RECEIVED.save(
                                deps.storage,
                                &from_json(result.get_query_result(0)?.1).unwrap(),
                            )?;
                        }
                        _ => unreachable!(),
                    }

                    Ok(Response::new().add_attribute("mock_callback_query", "executed"))
                }
                IbcResult::Execute { .. } => {
                    IBC_CALLBACK_RECEIVED.save(deps.storage, &true).unwrap();
                    Ok(Response::new().add_attribute("mock_callback", "executed"))
                }
                _ => unreachable!(),
            }
        })
        .with_replies(&[(1u64, |_, _, _, msg| {
            Ok(Response::new().set_data(msg.result.unwrap().data.unwrap()))
        })])
        .with_migrate(|_, _, _, _| Ok(Response::new().set_data("mock_migrate".as_bytes())))
        .with_module_ibc(|deps, _, _, src_module_info, _| {
            // We save the module info status
            MODULE_IBC_RECEIVED.save(deps.storage, &src_module_info.module)?;
            Ok(Response::new().add_attribute("mock_module_ibc", "executed"))
        })
        .with_dependencies(&[IBC_CLIENT_DEP])
}

pub mod origin_app {
    use abstract_testing::{module::TEST_MODULE_ID, TEST_VERSION};

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
        module: &MockAppRemoteI<MockBech32>,
        source_module_expected: Option<ModuleInfo>,
    ) -> AnyResult<()> {
        let source_module = module
            .get_received_ibc_module_status()
            .map(|s| s.received)?;

        assert_eq!(source_module, source_module_expected);
        Ok(())
    }

    fn assert_callback_status(module: &MockAppOriginI<MockBech32>, status: bool) -> AnyResult<()> {
        let get_received_ibc_callback_status_res: ReceivedIbcCallbackStatus =
            module.get_received_ibc_callback_status()?;

        assert_eq!(
            ReceivedIbcCallbackStatus { received: status },
            get_received_ibc_callback_status_res
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
        setup::{ibc_abstract_setup, mock_test::logger_test_init},
        JUNO, STARGAZE,
    };
    use abstract_app::objects::{module::ModuleInfo, TruncatedChainId};
    use abstract_interface::{
        AccountI, AccountQueryFns, AppDeployer, DeployStrategy, VCExecFns, VCQueryFns,
    };
    use abstract_std::account::{self, ModuleInstallConfig};
    use abstract_testing::{
        module::{TEST_MODULE_ID, TEST_NAMESPACE},
        TEST_VERSION,
    };
    use anyhow::Result as AnyResult;
    use cosmwasm_std::{coins, to_json_binary};
    use cw_orch::{environment::Environment, prelude::*};
    use cw_orch_interchain::{prelude::*, types::IbcPacketOutcome};

    #[test]
    fn target_module_must_exist() -> AnyResult<()> {
        logger_test_init();
        let mock_interchain =
            MockBech32InterchainEnv::new(vec![(JUNO, "juno"), (STARGAZE, "stargaze")]);

        let (abstr_origin, _abstr_remote) = ibc_abstract_setup(&mock_interchain, JUNO, STARGAZE)?;

        let remote_name = TruncatedChainId::from_chain_id(STARGAZE);

        let (origin_account, _remote_account_id) =
            create_test_remote_account(&abstr_origin, JUNO, STARGAZE, &mock_interchain, vec![])?;

        let app = MockAppOriginI::new(
            TEST_MODULE_ID,
            abstr_origin.version_control.environment().clone(),
        );

        abstr_origin.version_control.claim_namespace(
            origin_account.config()?.account_id,
            TEST_NAMESPACE.to_owned(),
        )?;

        app.deploy(TEST_VERSION.parse()?, DeployStrategy::Try)?;

        origin_account.install_app(&app, &MockInitMsg {}, &[])?;

        // The user on origin chain wants to change the account description
        let target_module_info =
            ModuleInfo::from_id(TEST_MODULE_ID_REMOTE, TEST_VERSION_REMOTE.into())?;
        let ibc_action_result = app.do_something_ibc(remote_name, target_module_info.clone())?;

        let ibc_result = mock_interchain.await_packets(JUNO, ibc_action_result)?;

        let expected_error_outcome = format!(
            "Module {} does not have a stored module reference",
            target_module_info
        );
        match &ibc_result.packets[0].outcome {
            IbcPacketOutcome::Timeout { .. } => {
                panic!("Expected a failed ack not a timeout !")
            }
            IbcPacketOutcome::Success { ack, .. } => assert!(String::from_utf8_lossy(ack)
                .to_string()
                .contains(&expected_error_outcome)),
        }

        Ok(())
    }

    #[test]
    fn target_account_must_have_module_installed() -> AnyResult<()> {
        logger_test_init();
        let mock_interchain =
            MockBech32InterchainEnv::new(vec![(JUNO, "juno"), (STARGAZE, "stargaze")]);

        let (abstr_origin, abstr_remote) = ibc_abstract_setup(&mock_interchain, JUNO, STARGAZE)?;

        let remote_name = TruncatedChainId::from_chain_id(STARGAZE);

        let (origin_account, _remote_account_id) =
            create_test_remote_account(&abstr_origin, JUNO, STARGAZE, &mock_interchain, vec![])?;

        let (remote_account, _remote_account_id) =
            create_test_remote_account(&abstr_remote, STARGAZE, JUNO, &mock_interchain, vec![])?;

        // Install local app
        let app = MockAppOriginI::new(
            TEST_MODULE_ID,
            abstr_origin.version_control.environment().clone(),
        );

        abstr_origin
            .version_control
            .claim_namespace(origin_account.id()?, TEST_NAMESPACE.to_owned())?;

        app.deploy(TEST_VERSION.parse()?, DeployStrategy::Try)?;

        origin_account.install_app(&app, &MockInitMsg {}, &[])?;

        // Install remote app
        let app_remote = MockAppRemoteI::new(
            TEST_MODULE_ID_REMOTE,
            abstr_remote.version_control.environment().clone(),
        );

        abstr_remote
            .version_control
            .claim_namespace(remote_account.id()?, TEST_NAMESPACE.to_owned())?;

        app_remote.deploy(TEST_VERSION_REMOTE.parse()?, DeployStrategy::Try)?;

        // The user on origin chain wants to change the account description
        let target_module_info =
            ModuleInfo::from_id(TEST_MODULE_ID_REMOTE, TEST_VERSION_REMOTE.into())?;
        let ibc_action_result = app.do_something_ibc(remote_name, target_module_info.clone())?;

        let ibc_result = mock_interchain.await_packets(JUNO, ibc_action_result)?;

        let expected_error_outcome =
            format!("App {} not installed on Account", target_module_info,);
        match &ibc_result.packets[0].outcome {
            IbcPacketOutcome::Timeout { .. } => {
                panic!("Expected a failed ack not a timeout !")
            }
            IbcPacketOutcome::Success { ack, .. } => assert!(String::from_utf8_lossy(ack)
                .to_string()
                .contains(&expected_error_outcome)),
        }

        Ok(())
    }

    #[test]
    fn works() -> AnyResult<()> {
        logger_test_init();
        let mock_interchain =
            MockBech32InterchainEnv::new(vec![(JUNO, "juno"), (STARGAZE, "stargaze")]);

        let (abstr_origin, abstr_remote) = ibc_abstract_setup(&mock_interchain, JUNO, STARGAZE)?;

        let remote_name = TruncatedChainId::from_chain_id(STARGAZE);

        let (origin_account, remote_account_id) =
            create_test_remote_account(&abstr_origin, JUNO, STARGAZE, &mock_interchain, vec![])?;

        let (remote_account, _) =
            create_test_remote_account(&abstr_remote, STARGAZE, JUNO, &mock_interchain, vec![])?;

        // Install local app
        let app = MockAppOriginI::new(
            TEST_MODULE_ID,
            abstr_origin.version_control.environment().clone(),
        );

        abstr_origin
            .version_control
            .claim_namespace(origin_account.id()?, TEST_NAMESPACE.to_owned())?;

        app.deploy(TEST_VERSION.parse()?, DeployStrategy::Try)?;

        origin_account.install_app(&app, &MockInitMsg {}, &[])?;

        // Install remote app
        let app_remote = MockAppRemoteI::new(
            TEST_MODULE_ID_REMOTE,
            abstr_remote.version_control.environment().clone(),
        );

        abstr_remote
            .version_control
            .claim_namespace(remote_account.id()?, TEST_NAMESPACE.to_owned())?;

        app_remote.deploy(TEST_VERSION_REMOTE.parse()?, DeployStrategy::Try)?;

        let remote_install_response = origin_account.execute_on_remote(
            remote_name.clone(),
            account::ExecuteMsg::InstallModules {
                modules: vec![ModuleInstallConfig::new(
                    ModuleInfo::from_id_latest(TEST_MODULE_ID_REMOTE)?,
                    Some(to_json_binary(&MockInitMsg {})?),
                )],
            },
        )?;

        mock_interchain.await_and_check_packets(JUNO, remote_install_response)?;

        // We get the object for handling the actual module on the remote account
        let remote_account = abstr_remote
            .version_control
            .account(remote_account_id)?
            .account_base;
        let account = AccountI::new(
            "remote-account-manager",
            abstr_remote.version_control.environment().clone(),
        );
        account.set_address(remote_account.addr());
        let module_address = account.module_info(TEST_MODULE_ID_REMOTE)?.unwrap().address;
        let remote_account_app = MockAppRemoteI::new(
            "remote-account-app",
            abstr_remote.version_control.environment().clone(),
        );
        remote_account_app.set_address(&module_address);

        // The user on origin chain triggers a module-to-module interaction
        let target_module_info =
            ModuleInfo::from_id(TEST_MODULE_ID_REMOTE, TEST_VERSION_REMOTE.into())?;
        let ibc_action_result =
            app.do_something_ibc(remote_name.clone(), target_module_info.clone())?;

        let source_module = app.get_received_ibc_module_status().map(|s| s.received)?;

        assert_eq!(source_module, None);

        let get_received_ibc_callback_status_res: ReceivedIbcCallbackStatus =
            app.get_received_ibc_callback_status()?;

        assert_eq!(
            ReceivedIbcCallbackStatus { received: false },
            get_received_ibc_callback_status_res
        );

        mock_interchain.await_and_check_packets(JUNO, ibc_action_result)?;

        assert_remote_module_call_status(
            &remote_account_app,
            Some(ModuleInfo::from_id(TEST_MODULE_ID, TEST_VERSION.into())?),
        )?;
        assert_callback_status(&app, true)?;

        // Module to module query

        let ibc_action_result = app.query_module_ibc(remote_name, target_module_info)?;
        mock_interchain.await_and_check_packets(JUNO, ibc_action_result)?;

        let status = app.get_received_module_ibc_query_callback_status()?;
        assert_eq!("bar", status);
        Ok(())
    }

    pub const REMOTE_AMOUNT: u128 = 5674309;
    pub const REMOTE_DENOM: &str = "remote_denom";

    #[test]
    fn queries() -> AnyResult<()> {
        logger_test_init();
        let mock_interchain =
            MockBech32InterchainEnv::new(vec![(JUNO, "juno"), (STARGAZE, "stargaze")]);

        let (abstr_origin, _abstr_remote) = ibc_abstract_setup(&mock_interchain, JUNO, STARGAZE)?;

        let remote_name = TruncatedChainId::from_chain_id(STARGAZE);
        let remote = mock_interchain.get_chain(STARGAZE)?;
        let remote_address =
            remote.addr_make_with_balance("remote-test", coins(REMOTE_AMOUNT, REMOTE_DENOM))?;

        let (origin_account, _remote_account_id) =
            create_test_remote_account(&abstr_origin, JUNO, STARGAZE, &mock_interchain, vec![])?;

        // Install local app
        let app = MockAppOriginI::new(
            TEST_MODULE_ID,
            abstr_origin.version_control.environment().clone(),
        );

        abstr_origin
            .version_control
            .claim_namespace(origin_account.id()?, TEST_NAMESPACE.to_owned())?;

        app.deploy(TEST_VERSION.parse()?, DeployStrategy::Try)?;

        origin_account.install_app(&app, &MockInitMsg {}, &[])?;

        let query_response = app.query_something_ibc(remote_address.to_string(), remote_name)?;

        let get_received_ibc_query_callback_status_res: ReceivedIbcQueryCallbackStatus =
            app.get_received_ibc_query_callback_status().unwrap();

        assert_eq!(
            ReceivedIbcQueryCallbackStatus { balance: vec![] },
            get_received_ibc_query_callback_status_res
        );

        mock_interchain.await_and_check_packets(JUNO, query_response)?;

        let get_received_ibc_query_callback_status_res: ReceivedIbcQueryCallbackStatus =
            app.get_received_ibc_query_callback_status().unwrap();

        assert_eq!(
            ReceivedIbcQueryCallbackStatus {
                balance: coins(REMOTE_AMOUNT, REMOTE_DENOM)
            },
            get_received_ibc_query_callback_status_res
        );

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

            let (abstr_origin, abstr_remote) =
                ibc_abstract_setup(&mock_interchain, JUNO, STARGAZE)?;

            let remote_name = TruncatedChainId::from_chain_id(STARGAZE);

            let (origin_account, remote_account_id) = create_test_remote_account(
                &abstr_origin,
                JUNO,
                STARGAZE,
                &mock_interchain,
                vec![],
            )?;

            let (remote_account, _) = create_test_remote_account(
                &abstr_remote,
                STARGAZE,
                JUNO,
                &mock_interchain,
                vec![],
            )?;

            // Install local app
            let app = MockAppOriginI::new(
                TEST_MODULE_ID,
                abstr_origin.version_control.environment().clone(),
            );

            abstr_origin
                .version_control
                .claim_namespace(origin_account.id()?, TEST_NAMESPACE.to_owned())?;

            app.deploy(TEST_VERSION.parse()?, DeployStrategy::Try)?;

            origin_account.install_app(&app, &MockInitMsg {}, &[])?;

            // Install remote app
            let app_remote = MockAppRemoteI::new(
                TEST_MODULE_ID_REMOTE,
                abstr_remote.version_control.environment().clone(),
            );

            abstr_remote
                .version_control
                .claim_namespace(remote_account.id()?, TEST_NAMESPACE.to_owned())?;

            app_remote.deploy(TEST_VERSION_REMOTE.parse()?, DeployStrategy::Try)?;

            let remote_install_response = origin_account.execute_on_remote(
                remote_name.clone(),
                account::ExecuteMsg::InstallModules {
                    modules: vec![ModuleInstallConfig::new(
                        ModuleInfo::from_id_latest(TEST_MODULE_ID_REMOTE)?,
                        Some(to_json_binary(&MockInitMsg {})?),
                    )],
                },
            )?;

            mock_interchain.await_and_check_packets(JUNO, remote_install_response)?;

            // We get the object for handling the actual module on the remote account
            let remote_account = abstr_remote
                .version_control
                .account(remote_account_id)?
                .account_base;
            let account = AccountI::new(
                "remote-account-manager",
                abstr_remote.version_control.environment().clone(),
            );
            account.set_address(remote_account.addr());
            let module_address = account.module_info(TEST_MODULE_ID_REMOTE)?.unwrap().address;
            let remote_account_app = MockAppRemoteI::new(
                "remote-account-app",
                abstr_remote.version_control.environment().clone(),
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
