use abstract_adapter::mock::MockInitMsg;
use abstract_ibc_host::HostError;
use abstract_interface::{
    Abstract, AdapterDeployer, DeployStrategy, ExecuteMsgFns as InterfaceExecuteMsgFns,
};
use abstract_std::{
    account::ModuleInstallConfig,
    ibc_host::{
        ClientProxyResponse, ConfigResponse, ExecuteMsgFns, HostAction, InternalAction, QueryMsgFns,
    },
    objects::{
        gov_type::{GovAction, GovernanceDetails},
        module::ModuleInfo,
        AccountId, AccountTrace, TruncatedChainId, UncheckedChannelEntry,
    },
    ACCOUNT, ICS20, VERSION_CONTROL,
};
use abstract_testing::prelude::mock_bech32_admin;
use cosmwasm_std::Event;
use cw_orch::prelude::*;
use cw_ownable::OwnershipError;

use crate::mock_adapter::{MockAdapter, MOCK_ADAPTER_ID};

mod mock_adapter {
    use abstract_adapter::gen_adapter_mock;

    use super::*;

    pub const MOCK_ADAPTER_ID: &str = "abstract:mock-adapter";
    gen_adapter_mock!(MockAdapter, MOCK_ADAPTER_ID, "1.0.0", &[]);
}

#[test]
fn account_creation() -> anyhow::Result<()> {
    let chain = MockBech32::new("mock");
    let sender = chain.sender_addr();

    let admin = mock_bech32_admin(&chain);
    let mut origin_chain = chain.clone();
    origin_chain.set_sender(admin.clone());

    let abstr_origin = Abstract::deploy_on(origin_chain.clone(), admin.clone())?;
    let abstr_remote = Abstract::load_from(chain.clone())?;

    let account_sequence = 1;
    let chain = "juno";

    // Verify config
    let config_response: ConfigResponse = abstr_origin.ibc.host.config()?;

    assert_eq!(
        ConfigResponse {
            ans_host_address: abstr_origin.ans_host.address()?,
            version_control_address: abstr_origin.version_control.address()?,
            module_factory_address: abstr_origin.module_factory.address()?,
        },
        config_response
    );

    // We need to set the sender as the proxy for juno chain
    abstr_origin
        .ibc
        .host
        .register_chain_proxy(chain.parse()?, sender.to_string())?;

    // Verify chain proxy via query
    let client_proxy_response: ClientProxyResponse =
        abstr_origin.ibc.host.client_proxy(chain.to_owned())?;

    assert_eq!(sender, client_proxy_response.proxy);

    // We call the account creation
    let account_creation_response = abstr_remote
        .ibc
        .host
        .ibc_execute(
            "proxy_address",
            AccountId::local(account_sequence),
            HostAction::Internal(InternalAction::Register {
                name: "Abstract remote account 1".to_string(),
                description: Some("account description".to_string()),
                link: Some("https://abstract.money".to_string()),
                namespace: None,
                install_modules: vec![],
            }),
        )
        .unwrap();

    assert!(account_creation_response.has_event(
        &Event::new("wasm-abstract")
            .add_attribute("_contract_address", abstr_remote.version_control.address()?)
            .add_attribute("contract", VERSION_CONTROL)
            .add_attribute("action", "add_account")
            .add_attribute(
                "account_id",
                AccountId::new(
                    account_sequence,
                    AccountTrace::Remote(vec![TruncatedChainId::from_chain_id(chain)])
                )?
                .to_string()
            )
    ));

    Ok(())
}

#[test]
fn cannot_register_proxy_as_non_owner() -> anyhow::Result<()> {
    let chain = MockBech32::new("mock");
    let sender = chain.sender_addr();

    let admin = mock_bech32_admin(&chain);
    let mut origin_chain = chain.clone();
    origin_chain.set_sender(admin.clone());

    let abstr_origin = Abstract::deploy_on(origin_chain.clone(), admin.clone())?;

    let chain_name = "juno";

    let err: CwOrchError = abstr_origin
        .ibc
        .host
        .call_as(&chain.addr_make("user"))
        .register_chain_proxy(chain_name.parse().unwrap(), sender.to_string())
        .unwrap_err();

    assert_eq!(
        HostError::OwnershipError(OwnershipError::NotOwner),
        err.downcast()?
    );

    Ok(())
}

#[test]
fn cannot_remove_proxy_as_non_owner() -> anyhow::Result<()> {
    let chain = MockBech32::new("mock");

    let admin = mock_bech32_admin(&chain);
    let mut origin_chain = chain.clone();
    origin_chain.set_sender(admin.clone());

    let abstr_origin = Abstract::deploy_on(origin_chain.clone(), admin.clone())?;

    let chain_name = "juno";

    let err: CwOrchError = abstr_origin
        .ibc
        .host
        .call_as(&chain.addr_make("user"))
        .remove_chain_proxy(chain_name.parse().unwrap())
        .unwrap_err();

    assert_eq!(
        HostError::OwnershipError(OwnershipError::NotOwner),
        err.downcast()?
    );

    Ok(())
}

#[test]
fn account_creation_full() -> anyhow::Result<()> {
    let chain = MockBech32::new("mock");
    let sender = chain.sender_addr();

    let admin = mock_bech32_admin(&chain);
    let mut origin_chain = chain.clone();
    origin_chain.set_sender(admin.clone());

    let abstr_origin = Abstract::deploy_on(origin_chain.clone(), admin.clone())?;
    let abstr_remote = Abstract::load_from(chain.clone())?;

    let account_sequence = 1;
    let chain_name = "juno";

    // Verify config
    let config_response: ConfigResponse = abstr_origin.ibc.host.config()?;

    assert_eq!(
        ConfigResponse {
            ans_host_address: abstr_origin.ans_host.address()?,
            version_control_address: abstr_origin.version_control.address()?,
            module_factory_address: abstr_origin.module_factory.address()?,
        },
        config_response
    );

    // We need to set the sender as the proxy for juno chain
    abstr_origin
        .ibc
        .host
        .register_chain_proxy(chain_name.parse().unwrap(), sender.to_string())?;

    // Add asset to set base_asset
    abstr_origin.ans_host.update_asset_addresses(
        vec![("juno>juno".to_owned(), "native:juno".parse().unwrap())],
        vec![],
    )?;

    // Verify chain proxy via query
    let client_proxy_response: ClientProxyResponse =
        abstr_origin.ibc.host.client_proxy(chain_name.to_owned())?;

    assert_eq!(sender, client_proxy_response.proxy);

    // Deploy app to install it during account registration
    let mock_adapter = MockAdapter::new_test(chain);
    mock_adapter.call_as(&admin).deploy(
        "1.0.0".parse().unwrap(),
        MockInitMsg {},
        DeployStrategy::Try,
    )?;

    let mock_module_install_config =
        ModuleInstallConfig::new(ModuleInfo::from_id_latest(MOCK_ADAPTER_ID).unwrap(), None);
    let account_creation_response = abstr_remote
        .ibc
        .host
        .ibc_execute(
            "proxy_address",
            AccountId::local(account_sequence),
            HostAction::Internal(InternalAction::Register {
                name: "Abstract remote account 1".to_string(),
                description: Some("account description".to_string()),
                link: Some("https://abstract.money".to_string()),
                namespace: Some("namespace".to_owned()),
                install_modules: vec![mock_module_install_config],
            }),
        )
        .unwrap();

    assert!(account_creation_response.has_event(
        &Event::new("wasm-abstract")
            .add_attribute("_contract_address", abstr_remote.version_control.address()?)
            .add_attribute("contract", VERSION_CONTROL)
            .add_attribute("action", "add_account")
            .add_attribute(
                "account_id",
                AccountId::new(
                    account_sequence,
                    AccountTrace::Remote(vec![TruncatedChainId::from_chain_id(chain_name)])
                )?
                .to_string()
            )
    ));

    Ok(())
}

#[test]
fn account_action() -> anyhow::Result<()> {
    let mock = MockBech32::new("mock");
    let sender = mock.sender().clone();

    let admin = mock_bech32_admin(&mock);
    let mut origin_chain = mock.clone();
    origin_chain.set_sender(admin.clone());

    let abstr_origin = Abstract::deploy_on(origin_chain.clone(), admin.clone())?;
    let abstr_remote = Abstract::load_from(mock.clone())?;

    let account_sequence = 1;
    let chain = "juno";

    // We need to set the sender as the proxy for juno chain
    abstr_origin
        .ibc
        .host
        .register_chain_proxy(chain.parse().unwrap(), sender.to_string())?;

    // We create the account
    let proxy_addr = mock.addr_make("proxy_address");
    abstr_remote
        .ibc
        .host
        .ibc_execute(
            proxy_addr.to_string(),
            AccountId::local(account_sequence),
            HostAction::Internal(InternalAction::Register {
                name: "Abstract remote account 1".to_string(),
                description: Some("account description".to_string()),
                link: Some("https://abstract.money".to_string()),
                namespace: None,
                install_modules: vec![],
            }),
        )
        .unwrap();

    // We call the action
    let account_action_response = abstr_remote
        .ibc
        .host
        .ibc_execute(
            proxy_addr,
            AccountId::local(account_sequence),
            HostAction::Dispatch {
                account_msgs: vec![abstract_std::account::ExecuteMsg::UpdateOwnership(
                    GovAction::TransferOwnership {
                        new_owner: GovernanceDetails::Monarchy {
                            monarch: mock.addr_make("new_owner").to_string(),
                        },
                        expiry: None,
                    },
                )],
            },
        )
        .unwrap();

    assert!(account_action_response.has_event(
        &Event::new("wasm-abstract")
            .add_attribute("contract", ACCOUNT)
            .add_attribute("action", "update_ownership")
            .add_attribute("owner", "abstract-ibc")
            .add_attribute("pending_owner", "monarch")
            .add_attribute("pending_expiry", "none")
    ));

    Ok(())
}

#[test]
fn execute_action_with_account_creation() -> anyhow::Result<()> {
    let mock = MockBech32::new("mock");
    let admin = mock_bech32_admin(&mock);

    let abstr = Abstract::deploy_on(mock.clone(), admin.clone())?;

    let account_sequence = 1;
    let chain = "juno";

    // We need to set the sender as the proxy for juno chain
    abstr
        .ibc
        .host
        .call_as(&admin)
        .register_chain_proxy(chain.parse().unwrap(), admin.to_string())?;

    // We call the action
    let account_action_response = abstr
        .ibc
        .host
        .call_as(&admin)
        .ibc_execute(
            mock.addr_make("proxy_address"),
            AccountId::local(account_sequence),
            HostAction::Dispatch {
                account_msgs: vec![abstract_std::account::ExecuteMsg::UpdateOwnership(
                    GovAction::TransferOwnership {
                        new_owner: GovernanceDetails::Monarchy {
                            monarch: mock.addr_make("new_owner").to_string(),
                        },
                        expiry: None,
                    },
                )],
            },
        )
        .unwrap();

    assert!(account_action_response.has_event(
        &Event::new("wasm-abstract")
            .add_attribute("contract", ACCOUNT)
            .add_attribute("action", "update_ownership")
            .add_attribute("owner", "abstract-ibc")
            .add_attribute("pending_owner", "monarch")
            .add_attribute("pending_expiry", "none")
    ));

    Ok(())
}

#[test]
fn execute_send_all_back_action() -> anyhow::Result<()> {
    let mock = MockBech32::new("mock");
    let admin = mock_bech32_admin(&mock);

    let abstr = Abstract::deploy_on(mock.clone(), admin.clone())?;

    let account_sequence = 1;
    let chain = "juno";

    let polytone_proxy = mock.addr_make("polytone_proxy");

    // We need to set the sender as the proxy for juno chain
    abstr
        .ibc
        .host
        .call_as(&admin)
        .register_chain_proxy(chain.parse().unwrap(), polytone_proxy.to_string())?;

    // Add the juno token ics20 channel.
    abstr.ans_host.call_as(&admin).update_channels(
        vec![(
            UncheckedChannelEntry {
                connected_chain: chain.to_owned(),
                protocol: ICS20.to_owned(),
            },
            String::from("juno"),
        )],
        vec![],
    )?;

    let proxy_addr = mock.addr_make("proxy_address");
    // We create the account
    abstr.ibc.host.call_as(&polytone_proxy).ibc_execute(
        proxy_addr.to_string(),
        AccountId::local(account_sequence),
        HostAction::Internal(InternalAction::Register {
            name: "Abstract remote account 1".to_string(),
            description: Some("account description".to_string()),
            link: Some("https://abstract.money".to_string()),
            namespace: None,
            install_modules: vec![],
        }),
    )?;

    // We call the action and verify that it completes without issues.
    let account_action_response = abstr.ibc.host.call_as(&polytone_proxy).ibc_execute(
        proxy_addr.to_string(),
        AccountId::local(account_sequence),
        HostAction::Helpers(abstract_std::ibc_host::HelperAction::SendAllBack {}),
    )?;

    // Possible to verify that funds have been sent?
    assert!(account_action_response.has_event(
        &Event::new("wasm-abstract")
            .add_attribute("contract", ACCOUNT)
            .add_attribute("action", "execute_module_action")
    ));

    Ok(())
}
