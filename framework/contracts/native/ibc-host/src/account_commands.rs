use abstract_sdk::{
    std::{objects::ChannelEntry, ICS20},
    Resolve,
};
use abstract_std::{
    account_factory,
    ibc_host::state::CONFIG,
    manager::{self, ModuleInstallConfig},
    objects::{AccountId, AssetEntry, TruncatedChainId},
    proxy,
    version_control::AccountBase,
    PROXY,
};
use cosmwasm_std::{
    to_json_binary, wasm_execute, CosmosMsg, Deps, DepsMut, Env, IbcMsg, Response, SubMsg,
};

use crate::{
    contract::{HostResponse, HostResult},
    endpoints::reply::{INIT_BEFORE_ACTION_REPLY_ID, RESPONSE_REPLY_ID},
    HostError,
};

// one hour
const PACKET_LIFETIME: u64 = 60 * 60;

/// Creates and registers proxy for remote Account
#[allow(clippy::too_many_arguments)]
pub fn receive_register(
    deps: DepsMut,
    env: Env,
    account_id: AccountId,
    name: String,
    description: Option<String>,
    link: Option<String>,
    base_asset: Option<AssetEntry>,
    namespace: Option<String>,
    install_modules: Vec<ModuleInstallConfig>,
    with_reply: bool,
) -> HostResult {
    let cfg = CONFIG.load(deps.storage)?;

    // verify that the origin last chain is the chain related to this channel, and that it is not `Local`
    account_id.trace().verify_remote()?;

    // create the message to instantiate the remote account
    let factory_msg = wasm_execute(
        cfg.account_factory,
        &account_factory::ExecuteMsg::CreateAccount {
            governance: abstract_std::objects::gov_type::GovernanceDetails::External {
                governance_address: env.contract.address.into_string(),
                governance_type: "abstract-ibc".into(), // at least 4 characters
            },
            name,
            description,
            link,
            // provide the origin chain id
            account_id: Some(account_id.clone()),

            base_asset,
            install_modules,
            namespace,
        },
        vec![],
    )?;

    // If we were ordered to have a reply after account creation
    let sub_msg = if with_reply {
        SubMsg::reply_on_success(factory_msg, INIT_BEFORE_ACTION_REPLY_ID)
    } else {
        SubMsg::new(factory_msg)
    };

    Ok(Response::new()
        .add_submessage(sub_msg)
        .add_attribute("action", "register"))
}

/// Execute manager message on local manager.
pub fn receive_dispatch(
    _deps: DepsMut,
    account: AccountBase,
    manager_msgs: Vec<manager::ExecuteMsg>,
) -> HostResult {
    // execute the message on the manager
    let msgs = manager_msgs
        .into_iter()
        .map(|msg| wasm_execute(&account.manager, &msg, vec![]))
        .collect::<Result<Vec<_>, _>>()?;

    let response = Response::new()
        .add_attribute("action", "receive_dispatch")
        // This is used to forward the data of the calling message
        // This means that only the last present data of will be forwarded
        .add_submessages(
            msgs.into_iter()
                .map(|m| SubMsg::reply_on_success(m.clone(), RESPONSE_REPLY_ID)),
        );

    Ok(response)
}

/// processes PacketMsg::SendAllBack variant
pub fn receive_send_all_back(
    deps: DepsMut,
    env: Env,
    account: AccountBase,
    client_proxy_address: String,
    client_chain: TruncatedChainId,
) -> HostResult {
    let wasm_msg = send_all_back(
        deps.as_ref(),
        env,
        account,
        client_proxy_address,
        client_chain,
    )?;

    Ok(HostResponse::action("receive_dispatch").add_message(wasm_msg))
}

/// construct the msg to send all the assets back
pub fn send_all_back(
    deps: Deps,
    env: Env,
    account: AccountBase,
    client_proxy_address: String,
    client_chain: TruncatedChainId,
) -> Result<CosmosMsg, HostError> {
    // get the ICS20 channel information
    let ans = CONFIG.load(deps.storage)?.ans_host;
    let ics20_channel_entry = ChannelEntry {
        connected_chain: client_chain,
        protocol: ICS20.to_string(),
    };
    let ics20_channel_id = ics20_channel_entry.resolve(&deps.querier, &ans)?;
    // get all the coins for the account
    let coins = deps.querier.query_all_balances(account.proxy)?;
    // Construct ics20 messages to send all the coins back
    let mut msgs: Vec<CosmosMsg> = vec![];
    for coin in coins {
        msgs.push(
            IbcMsg::Transfer {
                channel_id: ics20_channel_id.clone(),
                to_address: client_proxy_address.to_string(),
                amount: coin,
                timeout: env.block.time.plus_seconds(PACKET_LIFETIME).into(),
            }
            .into(),
        )
    }
    // call the message to send everything back through the manager
    let manager_msg = wasm_execute(
        account.manager,
        &manager::ExecuteMsg::ExecOnModule {
            module_id: PROXY.into(),
            exec_msg: to_json_binary(&proxy::ExecuteMsg::ModuleAction { msgs })?,
        },
        vec![],
    )?;
    Ok(manager_msg.into())
}

/// get the account base from the version control contract
pub fn get_account(deps: Deps, account_id: &AccountId) -> Result<AccountBase, HostError> {
    let version_control = CONFIG.load(deps.storage)?.version_control;
    let account_base = version_control.account_base(account_id, &deps.querier)?;
    Ok(account_base)
}
