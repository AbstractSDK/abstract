use abstract_sdk::{
    feature_objects::{AnsHost, VersionControlContract},
    std::{objects::ChannelEntry, ICS20},
    Resolve,
};
use abstract_std::{
    account::{self, ModuleInstallConfig},
    objects::{module::ModuleInfo, module_reference::ModuleReference, AccountId, TruncatedChainId},
    version_control::Account,
    ACCOUNT,
};
use cosmwasm_std::{
    instantiate2_address, to_json_binary, wasm_execute, CosmosMsg, Deps, DepsMut, Env, IbcMsg,
    Response, SubMsg, WasmMsg,
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
    namespace: Option<String>,
    install_modules: Vec<ModuleInstallConfig>,
    with_reply: bool,
) -> HostResult {
    let version_control = VersionControlContract::new(deps.api)?;
    // verify that the origin last chain is the chain related to this channel, and that it is not `Local`
    account_id.trace().verify_remote()?;
    let salt = cosmwasm_std::to_json_binary(&account_id)?;

    let account_module_info = ModuleInfo::from_id_latest(ACCOUNT)?;
    let ModuleReference::Account(code_id) = version_control
        .query_module(account_module_info.clone(), &deps.querier)?
        .reference
    else {
        return Err(HostError::VersionControlError(
            abstract_std::objects::version_control::VersionControlError::InvalidReference(
                account_module_info,
            ),
        ));
    };
    let checksum = deps.querier.query_wasm_code_info(code_id)?.checksum;
    let self_canon_addr = deps.api.addr_canonicalize(env.contract.address.as_str())?;

    let create_account_msg = account::InstantiateMsg::<cosmwasm_std::Empty> {
        owner: abstract_std::objects::gov_type::GovernanceDetails::External {
            governance_address: env.contract.address.into_string(),
            governance_type: "abstract-ibc".into(), // at least 4 characters
        },
        name,
        description,
        link,
        // provide the origin chain id
        account_id: Some(account_id.clone()),
        install_modules,
        namespace,
        authenticator: None,
    };

    let account_canon_addr =
        instantiate2_address(checksum.as_slice(), &self_canon_addr, salt.as_slice())?;
    let account_addr = deps.api.addr_humanize(&account_canon_addr)?;

    // create the message to instantiate the remote account
    let account_creation_message = WasmMsg::Instantiate2 {
        admin: Some(account_addr.to_string()),
        code_id,
        label: account_id.to_string(),
        msg: to_json_binary(&create_account_msg)?,
        funds: vec![],
        salt,
    };

    // If we were ordered to have a reply after account creation
    let sub_msg = if with_reply {
        SubMsg::reply_on_success(account_creation_message, INIT_BEFORE_ACTION_REPLY_ID)
    } else {
        SubMsg::new(account_creation_message)
    };

    Ok(Response::new()
        .add_submessage(sub_msg)
        .add_attribute("action", "register"))
}

/// Execute account message on local account.
pub fn receive_dispatch(
    _deps: DepsMut,
    account: Account,
    account_msgs: Vec<account::ExecuteMsg>,
) -> HostResult {
    // execute the message on the account
    let msgs = account_msgs
        .into_iter()
        .map(|msg| wasm_execute(account.addr(), &msg, vec![]))
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
    account: Account,
    client_proxy_address: String,
    src_chain: TruncatedChainId,
) -> HostResult {
    let wasm_msg = send_all_back(deps.as_ref(), env, account, client_proxy_address, src_chain)?;

    Ok(HostResponse::action("receive_dispatch").add_message(wasm_msg))
}

/// construct the msg to send all the assets back
pub fn send_all_back(
    deps: Deps,
    env: Env,
    account: Account,
    client_proxy_address: String,
    src_chain: TruncatedChainId,
) -> Result<CosmosMsg, HostError> {
    // get the ICS20 channel information
    let ans = AnsHost::new(deps.api)?;
    let ics20_channel_entry = ChannelEntry {
        connected_chain: src_chain,
        protocol: ICS20.to_string(),
    };
    let ics20_channel_id = ics20_channel_entry.resolve(&deps.querier, &ans)?;
    // get all the coins for the account
    let coins = deps.querier.query_all_balances(account.addr())?;
    // Construct ics20 messages to send all the coins back
    let mut msgs: Vec<CosmosMsg> = vec![];
    for coin in coins {
        msgs.push(
            IbcMsg::Transfer {
                channel_id: ics20_channel_id.clone(),
                to_address: client_proxy_address.to_string(),
                amount: coin,
                timeout: env.block.time.plus_seconds(PACKET_LIFETIME).into(),
                memo: None,
            }
            .into(),
        )
    }
    // call the message to send everything back through the account
    let account_msg = wasm_execute(
        account.into_addr(),
        &account::ExecuteMsg::ModuleAction::<cosmwasm_std::Empty> { msgs },
        vec![],
    )?;
    Ok(account_msg.into())
}

/// get the account from the version control contract
pub fn get_account(deps: Deps, account_id: &AccountId) -> Result<Account, HostError> {
    let version_control = VersionControlContract::new(deps.api)?;
    let account = version_control.account(account_id, &deps.querier)?;
    Ok(account)
}
