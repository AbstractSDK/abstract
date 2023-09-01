use abstract_core::ibc_host::ExecuteMsgFns;
use abstract_core::ibc_host::HostAction;
use abstract_core::ibc_host::InternalAction;
use abstract_core::objects::gov_type::GovernanceDetails;
use abstract_core::objects::AccountId;
use abstract_core::ACCOUNT_FACTORY;
use abstract_core::MANAGER;
use abstract_interface::Abstract;
use cosmwasm_std::Event;
use cw_orch::deploy::Deploy;

use cw_orch::prelude::*;


#[test]
fn account_creation() -> anyhow::Result<()> {
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

    // We call the account creation
    let account_creation_response = abstr
        .ibc
        .host
        .ibc_execute(
            AccountId::local(account_sequence),
            HostAction::Internal(InternalAction::Register{
                name: "Abstract remote account 1".to_string(),
                description: Some("account description".to_string()),
                link: Some("https://abstract.money".to_string())
            }), 
            "proxy_address".to_string(),
        )
        .unwrap();

    assert!(account_creation_response.has_event(
        &Event::new("wasm-abstract")
            .add_attribute("_contract_addr", abstr.account_factory.address()?)
            .add_attribute("contract", ACCOUNT_FACTORY)
            .add_attribute("action", "create_account")
            .add_attribute("account_sequence", account_sequence.to_string())
            .add_attribute("trace", chain)
    ));

    Ok(())
}


#[test]
fn account_action() -> anyhow::Result<()> {
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

    // We create the account
    abstr
        .ibc
        .host
        .ibc_execute(
            AccountId::local(account_sequence),
            HostAction::Internal(InternalAction::Register{
                name: "Abstract remote account 1".to_string(),
                description: Some("account description".to_string()),
                link: Some("https://abstract.money".to_string())
            }), 
            "proxy_address".to_string(),
        )
        .unwrap();

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

    assert!(!account_action_response.has_event(
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
