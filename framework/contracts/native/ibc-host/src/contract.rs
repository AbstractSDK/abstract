use crate::{
    endpoints::{
        self,
        reply::{
            reply_execute_action, reply_forward_response_data, INIT_BEFORE_ACTION_REPLY_ID,
            RESPONSE_REPLY_ID,
        },
    },
    error::HostError,
};
use abstract_core::{ibc_host::ExecuteMsg, IBC_HOST};
use abstract_macros::abstract_response;
use abstract_sdk::core::ibc_host::{InstantiateMsg, QueryMsg};
use cosmwasm_std::{
    Binary, Deps, DepsMut, Env, IbcReceiveResponse, MessageInfo, Reply, Response, StdError,
};

pub const CONTRACT_VERSION: &str = env!("CARGO_PKG_VERSION");

#[abstract_response(IBC_HOST)]
pub struct HostResponse;

pub type HostResult<T = Response> = Result<T, HostError>;
pub type IbcHostResult = HostResult<IbcReceiveResponse>;

#[cfg_attr(feature = "export", cosmwasm_std::entry_point)]
pub fn instantiate(deps: DepsMut, env: Env, info: MessageInfo, msg: InstantiateMsg) -> HostResult {
    endpoints::instantiate(deps, env, info, msg)
}

#[cfg_attr(feature = "export", cosmwasm_std::entry_point)]
pub fn execute(deps: DepsMut, env: Env, info: MessageInfo, msg: ExecuteMsg) -> HostResult {
    // will only process base requests as there is no exec handler set.
    endpoints::execute(deps, env, info, msg)
}

#[cfg_attr(feature = "export", cosmwasm_std::entry_point)]
pub fn query(deps: Deps, env: Env, msg: QueryMsg) -> HostResult<Binary> {
    // will only process base requests as there is no exec handler set.
    endpoints::query(deps, env, msg)
}

#[cfg_attr(feature = "export", cosmwasm_std::entry_point)]
pub fn reply(deps: DepsMut, env: Env, reply_msg: Reply) -> HostResult {
    if reply_msg.id == INIT_BEFORE_ACTION_REPLY_ID {
        reply_execute_action(deps, env, reply_msg)
    } else if reply_msg.id == RESPONSE_REPLY_ID {
        reply_forward_response_data(reply_msg)
    } else {
        Err(HostError::Std(StdError::generic_err("Not implemented")))
    }
}

#[cfg(test)]
mod test {
    use abstract_core::ibc_host::ClientProxyResponse;
    use abstract_core::ibc_host::ConfigResponse;
    use abstract_core::ibc_host::ExecuteMsgFns;
    use abstract_core::ibc_host::HostAction;
    use abstract_core::ibc_host::InternalAction;
    use abstract_core::ibc_host::QueryMsgFns;
    use abstract_core::objects::gov_type::GovernanceDetails;
    use abstract_core::objects::AccountId;
    use abstract_core::objects::UncheckedChannelEntry;
    use abstract_core::ACCOUNT_FACTORY;
    use abstract_core::ICS20;
    use abstract_core::MANAGER;
    use abstract_interface::Abstract;
    use abstract_interface::ExecuteMsgFns as InterfaceExecuteMsgFns;
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

        // Verify config
        let config_response: ConfigResponse = admin_abstr.ibc.host.config()?;

        assert_eq!(
            ConfigResponse {
                ans_host_address: admin_abstr.ans_host.address()?,
                version_control_address: admin_abstr.version_control.address()?,
                account_factory_address: admin_abstr.account_factory.address()?,
            },
            config_response
        );

        // We need to set the sender as the proxy for juno chain
        admin_abstr
            .ibc
            .host
            .register_chain_proxy(chain.into(), sender.to_string())?;

        // Verify chain proxy via query
        let client_proxy_response: ClientProxyResponse =
            admin_abstr.ibc.host.client_proxy(chain.to_owned())?;

        assert_eq!(sender, client_proxy_response.proxy);

        // We call the account creation
        let account_creation_response = abstr
            .ibc
            .host
            .ibc_execute(
                AccountId::local(account_sequence),
                HostAction::Internal(InternalAction::Register {
                    name: "Abstract remote account 1".to_string(),
                    description: Some("account description".to_string()),
                    link: Some("https://abstract.money".to_string()),
                }),
                "proxy_address".to_string(),
            )
            .unwrap();

        assert!(account_creation_response.has_event(
            &Event::new("wasm-abstract")
                .add_attribute("_contract_address", abstr.account_factory.address()?)
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
                HostAction::Internal(InternalAction::Register {
                    name: "Abstract remote account 1".to_string(),
                    description: Some("account description".to_string()),
                    link: Some("https://abstract.money".to_string()),
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
                .add_attribute("_contract_address", abstr.account_factory.address()?)
                .add_attribute("contract", ACCOUNT_FACTORY)
                .add_attribute("action", "create_account")
                .add_attribute("account_sequence", account_sequence.to_string())
                .add_attribute("trace", chain)
        ));

        assert!(account_action_response.has_event(
            &Event::new("wasm-abstract")
                .add_attribute("_contract_address", "contract9") // No simple way to get the account manager here, TODO ? For testing, we will keep that.
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
                .add_attribute("_contract_address", abstr.account_factory.address()?)
                .add_attribute("contract", ACCOUNT_FACTORY)
                .add_attribute("action", "create_account")
                .add_attribute("account_sequence", account_sequence.to_string())
                .add_attribute("trace", chain)
        ));

        assert!(account_action_response.has_event(
            &Event::new("wasm-abstract")
                .add_attribute("_contract_address", "contract9") // No simple way to get the account manager here, TODO ? For testing, we will keep that.
                .add_attribute("contract", MANAGER)
                .add_attribute("action", "update_owner")
                .add_attribute("governance_type", "monarch")
        ));

        Ok(())
    }

    #[test]
    fn execute_send_all_back_action() -> anyhow::Result<()> {
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

        // Add the juno token ics20 channel.
        admin_abstr.ans_host.update_channels(
            vec![(
                UncheckedChannelEntry {
                    connected_chain: chain.to_owned(),
                    protocol: ICS20.to_owned(),
                },
                String::from("juno"),
            )],
            vec![],
        )?;

        // We create the account
        abstr.ibc.host.ibc_execute(
            AccountId::local(account_sequence),
            HostAction::Internal(InternalAction::Register {
                name: "Abstract remote account 1".to_string(),
                description: Some("account description".to_string()),
                link: Some("https://abstract.money".to_string()),
            }),
            "proxy_address".to_string(),
        )?;

        // We call the action and verify that it completes without issues.
        abstr.ibc.host.ibc_execute(
            AccountId::local(account_sequence),
            HostAction::Helpers(abstract_core::ibc_host::HelperAction::SendAllBack {}),
            "proxy_address".to_string(),
        )?;

        Ok(())
    }
}
