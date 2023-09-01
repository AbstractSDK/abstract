use abstract_core::ibc_host::ExecuteMsgFns;
use abstract_core::ibc_host::HostAction;
use abstract_core::ibc_host::InstantiateMsg;
use abstract_core::ibc_host::QueryMsg;
use abstract_core::ibc_host::RegisteredChainResponse;
use abstract_core::objects::gov_type::GovernanceDetails;
use abstract_core::objects::AccountId;
use abstract_core::ACCOUNT_FACTORY;
use abstract_core::MANAGER;
use abstract_interface::Abstract;
use cosmwasm_std::from_binary;
use cosmwasm_std::testing::mock_dependencies;
use cosmwasm_std::testing::mock_env;
use cosmwasm_std::testing::mock_info;
use cosmwasm_std::Event;
use cw_orch::deploy::Deploy;

use crate::contract::execute;
use crate::contract::instantiate;
use crate::contract::query;
use cw_orch::prelude::*;

#[test]
fn test_registered_client() {
    // Instantiate
    let mut deps = mock_dependencies();
    let info = mock_info("admin", &[]);
    instantiate(
        deps.as_mut(),
        mock_env(),
        info.clone(),
        InstantiateMsg {
            account_factory_address: "dummy".to_string(),
            version_control_address: "foo".to_string(),
            ans_host_address: "bar".to_string(),
        },
    )
    .unwrap();

    // Register
    execute(
        deps.as_mut(),
        mock_env(),
        info,
        abstract_core::ibc_host::ExecuteMsg::RegisterChainProxy {
            chain: "juno".into(),
            proxy: "juno-proxy".to_string(),
        },
    )
    .unwrap();

    // Query
    let client_name = query(
        deps.as_ref(),
        mock_env(),
        QueryMsg::AssociatedClient {
            chain: "juno".to_string(),
        },
    )
    .unwrap();
    let queried_client_name: RegisteredChainResponse = from_binary(&client_name).unwrap();
    assert_eq!(queried_client_name.proxy, "juno-proxy");
}

#[test]
fn execute_action_with_account_creation() -> anyhow::Result<()> {
    let sender = Addr::unchecked("sender");
    let chain = Mock::new(&sender);

    let admin = Addr::unchecked("admin");
    let mut admin_chain = chain.clone();
    admin_chain.set_sender(admin.clone());

    let admin_abstr = Abstract::deploy_on(admin_chain.clone(), admin.to_string())?;
    let abstr = Abstract::load_from(chain.clone())?;

    let account_sequence = 1;
    let chain = "juno";

    // We need to set the sender as the proxy for juno chain
    admin_abstr
        .ibc
        .host
        .register_chain_proxy(chain.into(), sender.to_string())?;

    // We call the action
    let account_action_response = abstr
        .ibc
        .host
        .ibc_execute(
            AccountId::local(account_sequence),
            HostAction::Dispatch {
                manager_msg: abstract_core::manager::ExecuteMsg::SetOwner {
                    owner: GovernanceDetails::Monarchy {
                        monarch: "new_owner".to_string(),
                    },
                },
            },
            "proxy_address".to_string(),
        )
        .unwrap();

    assert!(account_action_response.has_event(
        &Event::new("wasm-abstract")
            .add_attribute("_contract_addr", abstr.account_factory.address()?)
            .add_attribute("contract", ACCOUNT_FACTORY)
            .add_attribute("action", "create_account")
            .add_attribute("account_sequence", account_sequence.to_string())
            .add_attribute("trace", chain)
    ));

    assert!(account_action_response.has_event(
        &Event::new("wasm-abstract")
            .add_attribute("_contract_addr", "contract9") // No simple way to get the account manager here, TODO, of for testing ?
            .add_attribute("contract", MANAGER)
            .add_attribute("action", "update_owner")
            .add_attribute("governance_type", "monarch")
    ));

    Ok(())
}
