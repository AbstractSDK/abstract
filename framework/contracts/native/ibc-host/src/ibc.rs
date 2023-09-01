use crate::{
    contract::HostResult,
    endpoints::reply::INIT_CALLBACK_ID,
};
use abstract_core::{account_factory, objects::AccountId, ibc_host::state::{CLIENT_PROXY, REGISTRATION_CACHE, CONFIG}};

use cosmwasm_std::{
    wasm_execute, DepsMut, Env, Response, SubMsg,
};

// processes PacketMsg::Register variant
/// Creates and registers proxy for remote Account
#[allow(clippy::too_many_arguments)]
pub fn receive_register(
    deps: DepsMut,
    env: Env,
    account_id: AccountId,
    account_proxy_address: String,
    name: String,
    description: Option<String>,
    link: Option<String>,
) -> HostResult {
    let cfg = CONFIG.load(deps.storage)?;

    // verify that the origin last chain is the chain related to this channel, and that it is not `Local`
    account_id.trace().verify_remote()?;

    // create the message to instantiate the remote account
    let factory_msg = wasm_execute(
        cfg.account_factory,
        &account_factory::ExecuteMsg::CreateAccount {
            governance: abstract_core::objects::gov_type::GovernanceDetails::External {
                governance_address: env.contract.address.into_string(),
                governance_type: "abstract-ibc".into(), // at least 4 characters
            },
            name,
            description,
            link,
            // provide the origin chain id
            account_id: Some(account_id.clone()),

            base_asset: None,
            install_modules: vec![],
            namespace: None,
        },
        vec![],
    )?;
    // wrap with a submsg
    let factory_msg = SubMsg::reply_on_success(factory_msg, INIT_CALLBACK_ID);

    // store the proxy address of the Account on the client chain.
    CLIENT_PROXY.save(deps.storage, &account_id, &account_proxy_address)?;
    // store the account info for the reply handler
    REGISTRATION_CACHE.save(deps.storage, &account_id.clone())?;

    Ok(Response::new()
        .add_submessage(factory_msg)
        .add_attribute("action", "register"))
}
