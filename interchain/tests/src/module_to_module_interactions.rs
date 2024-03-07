pub use abstract_core::app;
use abstract_core::{
    ibc::{CallbackInfo, ModuleIbcMsg},
    ibc_client::{self, InstalledModuleIdentification},
    manager::ModuleInstallConfig,
    objects::{dependency::StaticDependency, module::ModuleInfo},
    IBC_CLIENT,
};
use abstract_interface::{AppDeployer, DependencyCreation};
use cosmwasm_schema::{cw_serde, QueryResponses};
pub use cosmwasm_std::testing::{mock_dependencies, mock_env, mock_info};
use cosmwasm_std::{to_json_binary, wasm_execute, Response, StdError};
use cw_controllers::AdminError;
use cw_orch::prelude::*;
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
        remote_chain: String,
        target_module: InstalledModuleIdentification,
    },
}

#[cosmwasm_schema::cw_serde]
#[derive(cw_orch::QueryFns)]
#[impl_into(QueryMsg)]
#[derive(QueryResponses)]
pub enum MockQueryMsg {
    #[returns(ReceivedIbcCallbackStatus)]
    GetReceivedIbcCallbackStatus {},

    #[returns(ReceivedIbcModuleStatus)]
    GetReceivedIbcModuleStatus {},
}

#[cosmwasm_schema::cw_serde]
pub struct ReceivedIbcCallbackStatus {
    pub received: bool,
}

#[cosmwasm_schema::cw_serde]
pub struct ReceivedIbcModuleStatus {
    pub received: InstalledModuleIdentification,
}

#[cosmwasm_schema::cw_serde]
pub struct MockMigrateMsg;

#[cosmwasm_schema::cw_serde]
pub struct MockReceiveMsg;

#[cosmwasm_schema::cw_serde]
pub struct MockSudoMsg;

use abstract_sdk::{
    base::InstantiateEndpoint, features::AccountIdentification, AbstractSdkError, ModuleInterface,
};
use abstract_testing::{
    addresses::{test_account_base, TEST_ANS_HOST, TEST_VERSION_CONTROL},
    prelude::{
        MockDeps, MockQuerierBuilder, TEST_MODULE_FACTORY, TEST_MODULE_ID, TEST_VERSION,
        TEST_WITH_DEP_MODULE_ID,
    },
};
use thiserror::Error;

use abstract_app::{AppContract, AppError};

#[derive(Error, Debug, PartialEq)]
pub enum MockError {
    #[error("{0}")]
    Std(#[from] StdError),

    #[error("{0}")]
    DappError(#[from] AppError),

    #[error("{0}")]
    Abstract(#[from] abstract_core::AbstractError),

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
pub const MODULE_IBC_RECEIVED: Item<InstalledModuleIdentification> =
    Item::new("module_ibc_received");

pub const fn mock_app(id: &'static str, version: &'static str) -> MockAppContract {
    MockAppContract::new(id, version, None)
        .with_instantiate(|deps, _, _, _, _| {
            IBC_CALLBACK_RECEIVED.save(deps.storage, &false)?;
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
                        source_module: InstalledModuleIdentification {
                            module_info: ModuleInfo::from_id(app.module_id(), app.version().into())
                                .unwrap(),
                            account_id: Some(app.account_id(deps.as_ref())?),
                        },
                        target_module,
                        msg: to_json_binary(&IbcModuleToModuleMsg {
                            ibc_msg: "module_to_module:msg".to_string(),
                        })
                        .unwrap(),
                        callback_info: Some(CallbackInfo {
                            id: "c_id".to_string(),
                            msg: None,
                            receiver: env.contract.address.to_string(),
                        }),
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
        })
        .with_sudo(|_, _, _, _| Ok(Response::new().set_data("mock_sudo".as_bytes())))
        .with_receive(|_, _, _, _, _| Ok(Response::new().set_data("mock_receive".as_bytes())))
        .with_ibc_callbacks(&[("c_id", |deps, _, _, _, _, _, _| {
            IBC_CALLBACK_RECEIVED.save(deps.storage, &true).unwrap();
            Ok(Response::new().add_attribute("mock_callback", "executed"))
        })])
        .with_replies(&[(1u64, |_, _, _, msg| {
            Ok(Response::new().set_data(msg.result.unwrap().data.unwrap()))
        })])
        .with_migrate(|_, _, _, _| Ok(Response::new().set_data("mock_migrate".as_bytes())))
        .with_module_ibc(|deps, _, _, msg| {
            let ModuleIbcMsg { source_module, .. } = msg;
            // We save the module info status
            MODULE_IBC_RECEIVED.save(deps.storage, &source_module)?;
            Ok(Response::new())
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
        source_module_expected: Option<InstalledModuleIdentification>,
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
    use crate::{
        interchain_accounts::create_test_remote_account,
        module_to_module_interactions::{
            origin_app::interface::MockAppOriginI,
            remote_app::{interface::MockAppRemoteI, TEST_MODULE_ID_REMOTE, TEST_VERSION_REMOTE},
            MockExecMsgFns, MockInitMsg, MockQueryMsgFns, ReceivedIbcCallbackStatus,
        },
        setup::{
            ibc_abstract_setup, ibc_connect_polytone_and_abstract, mock_test::logger_test_init,
        },
        JUNO, STARGAZE,
    };
    use abstract_app::objects::{chain_name::ChainName, module::ModuleInfo, AccountId};
    use abstract_core::{
        ibc_client::InstalledModuleIdentification, manager::ModuleAddressesResponse,
    };
    use abstract_interface::{AppDeployer, DeployStrategy, ManagerQueryFns, VCExecFns};
    use abstract_testing::addresses::{TEST_MODULE_ID, TEST_NAMESPACE, TEST_VERSION};
    use anyhow::Result as AnyResult;
    use base64::{engine::general_purpose, Engine};
    use cw_orch::interchain::MockBech32InterchainEnv;
    use cw_orch::prelude::*;

    #[test]
    fn target_module_must_exist() -> AnyResult<()> {
        logger_test_init();
        let mock_interchain =
            MockBech32InterchainEnv::new(vec![(JUNO, "juno"), (STARGAZE, "stargaze")]);

        // We just verified all steps pass
        let (abstr_origin, abstr_remote) = ibc_abstract_setup(&mock_interchain, JUNO, STARGAZE)?;
        ibc_connect_polytone_and_abstract(&mock_interchain, STARGAZE, JUNO)?;

        let remote_name = ChainName::from_chain_id(STARGAZE).to_string();

        let (origin_account, _remote_account_id) =
            create_test_remote_account(&abstr_origin, JUNO, STARGAZE, &mock_interchain, None)?;

        let (remote_account, _remote_account_id) =
            create_test_remote_account(&abstr_remote, STARGAZE, JUNO, &mock_interchain, None)?;

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
        let res: ModuleAddressesResponse = origin_account
            .manager
            .module_addresses(vec![TEST_MODULE_ID.to_owned()])?;

        assert_eq!(1, res.modules.len());

        // The user on origin chain wants to change the account description
        let target_module_info =
            ModuleInfo::from_id(TEST_MODULE_ID_REMOTE, TEST_VERSION_REMOTE.into())?;
        let ibc_action_result = app.do_something_ibc(
            remote_name,
            InstalledModuleIdentification {
                module_info: target_module_info.clone(),
                account_id: Some(remote_account.id()?),
            },
        )?;

        let ibc_result = mock_interchain.wait_ibc(JUNO, ibc_action_result)?;

        let expected_error_outcome = format!(
            "Module {} does not have a stored module reference",
            target_module_info
        );
        match &ibc_result.packets[0].outcome {
            cw_orch::interchain::types::IbcPacketOutcome::Timeout { .. } => {
                panic!("Expected a failed ack not a timeout !")
            }
            cw_orch::interchain::types::IbcPacketOutcome::Success { ack, .. } => match ack {
                cw_orch::interchain::types::IbcPacketAckDecode::Error(e) => {
                    assert!(e.contains(&expected_error_outcome));
                }
                cw_orch::interchain::types::IbcPacketAckDecode::Success(_) => {
                    panic!("Expected a error ack")
                }
                cw_orch::interchain::types::IbcPacketAckDecode::NotParsed(original_ack) => {
                    let error_str =
                        String::from_utf8_lossy(&general_purpose::STANDARD.decode(original_ack)?)
                            .to_string();
                    assert!(error_str.contains(&expected_error_outcome));
                }
            },
        }

        Ok(())
    }

    #[test]
    fn target_account_must_exist() -> AnyResult<()> {
        logger_test_init();
        let mock_interchain =
            MockBech32InterchainEnv::new(vec![(JUNO, "juno"), (STARGAZE, "stargaze")]);

        // We just verified all steps pass
        let (abstr_origin, abstr_remote) = ibc_abstract_setup(&mock_interchain, JUNO, STARGAZE)?;
        ibc_connect_polytone_and_abstract(&mock_interchain, STARGAZE, JUNO)?;

        let remote_name = ChainName::from_chain_id(STARGAZE).to_string();

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
        let res: ModuleAddressesResponse = origin_account
            .manager
            .module_addresses(vec![TEST_MODULE_ID.to_owned()])?;

        assert_eq!(1, res.modules.len());

        // Install remote app
        let app_remote = MockAppRemoteI::new(
            TEST_MODULE_ID_REMOTE,
            abstr_remote.version_control.get_chain().clone(),
        );

        abstr_remote
            .version_control
            .claim_namespace(remote_account.id()?, TEST_NAMESPACE.to_owned())?;

        app_remote.deploy(TEST_VERSION_REMOTE.parse()?, DeployStrategy::Try)?;

        remote_account.install_app(&app_remote, &MockInitMsg {}, None)?;

        // The user on origin chain wants to change the account description
        let unknown_account_id = AccountId::local(remote_account.id()?.seq() + 80);
        let target_module_info =
            ModuleInfo::from_id(TEST_MODULE_ID_REMOTE, TEST_VERSION_REMOTE.into())?;
        let ibc_action_result = app.do_something_ibc(
            remote_name,
            InstalledModuleIdentification {
                module_info: target_module_info.clone(),
                // This account is not supposed to exist on the remote chain
                account_id: Some(unknown_account_id.clone()),
            },
        )?;

        let ibc_result = mock_interchain.wait_ibc(JUNO, ibc_action_result)?;

        let expected_error_outcome = format!("Unknown Account id {}", unknown_account_id);
        match &ibc_result.packets[0].outcome {
            cw_orch::interchain::types::IbcPacketOutcome::Timeout { .. } => {
                panic!("Expected a failed ack not a timeout !")
            }
            cw_orch::interchain::types::IbcPacketOutcome::Success { ack, .. } => match ack {
                cw_orch::interchain::types::IbcPacketAckDecode::Error(e) => {
                    assert!(e.contains(&expected_error_outcome));
                }
                cw_orch::interchain::types::IbcPacketAckDecode::Success(_) => {
                    panic!("Expected a error ack")
                }
                cw_orch::interchain::types::IbcPacketAckDecode::NotParsed(original_ack) => {
                    let error_str =
                        String::from_utf8_lossy(&general_purpose::STANDARD.decode(original_ack)?)
                            .to_string();
                    assert!(error_str.contains(&expected_error_outcome));
                }
            },
        }
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

        let remote_name = ChainName::from_chain_id(STARGAZE).to_string();

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
        let res: ModuleAddressesResponse = origin_account
            .manager
            .module_addresses(vec![TEST_MODULE_ID.to_owned()])?;

        assert_eq!(1, res.modules.len());

        let local_module_address = res.modules[0].1.to_string();

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
        let ibc_action_result = app.do_something_ibc(
            remote_name,
            InstalledModuleIdentification {
                module_info: target_module_info.clone(),
                // This account is not supposed to exist on the remote chain
                account_id: Some(remote_account.id()?),
            },
        )?;

        let ibc_result = mock_interchain.wait_ibc(JUNO, ibc_action_result)?;

        let expected_error_outcome = format!(
            "Missing module {} on account {}",
            target_module_info,
            remote_account.id()?
        );
        match &ibc_result.packets[0].outcome {
            cw_orch::interchain::types::IbcPacketOutcome::Timeout { timeout_tx } => {
                panic!("Expected a failed ack not a timeout !")
            }
            cw_orch::interchain::types::IbcPacketOutcome::Success {
                receive_tx,
                ack_tx,
                ack,
            } => match ack {
                cw_orch::interchain::types::IbcPacketAckDecode::Error(e) => {
                    assert!(e.contains(&expected_error_outcome));
                }
                cw_orch::interchain::types::IbcPacketAckDecode::Success(_) => {
                    panic!("Expected a error ack")
                }
                cw_orch::interchain::types::IbcPacketAckDecode::NotParsed(original_ack) => {
                    let error_str =
                        String::from_utf8_lossy(&general_purpose::STANDARD.decode(original_ack)?)
                            .to_string();
                    assert!(error_str.contains(&expected_error_outcome));
                }
            },
        }

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

        let remote_name = ChainName::from_chain_id(STARGAZE).to_string();

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
        let res: ModuleAddressesResponse = origin_account
            .manager
            .module_addresses(vec![TEST_MODULE_ID.to_owned()])?;

        assert_eq!(1, res.modules.len());

        // Install remote app
        let app_remote = MockAppRemoteI::new(
            TEST_MODULE_ID_REMOTE,
            abstr_remote.version_control.get_chain().clone(),
        );

        abstr_remote
            .version_control
            .claim_namespace(remote_account.id()?, TEST_NAMESPACE.to_owned())?;

        app_remote.deploy(TEST_VERSION_REMOTE.parse()?, DeployStrategy::Try)?;
        remote_account.install_app(&app_remote, &MockInitMsg {}, None)?;

        // The user on origin chain wants to change the account description
        let target_module_info =
            ModuleInfo::from_id(TEST_MODULE_ID_REMOTE, TEST_VERSION_REMOTE.into())?;
        let ibc_action_result = app.do_something_ibc(
            remote_name,
            InstalledModuleIdentification {
                module_info: target_module_info.clone(),
                // This account is not supposed to exist on the remote chain
                account_id: Some(remote_account.id()?),
            },
        )?;

        assert_remote_module_call_status(&app_remote, None)?;
        assert_callback_status(&app, false)?;

        mock_interchain.wait_ibc(JUNO, ibc_action_result)?;

        assert_remote_module_call_status(
            &app_remote,
            Some(InstalledModuleIdentification {
                module_info: ModuleInfo::from_id(TEST_MODULE_ID, TEST_VERSION.into())?,
                account_id: Some(origin_account.id()?),
            }),
        )?;
        assert_callback_status(&app, true)?;

        Ok(())
    }
}
