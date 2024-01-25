use abstract_core::objects::{account::AccountTrace, fee::UsageFee, AccountId};
use abstract_dex_standard::msg::DexInstantiateMsg;
use abstract_sdk::AccountVerification;
use cosmwasm_std::{DepsMut, Env, MessageInfo, Response};

use crate::{
    contract::{DexAdapter, DexResult},
    state::SWAP_FEE,
};

pub fn instantiate_handler(
    deps: DepsMut,
    _env: Env,
    _info: MessageInfo,
    adapter: DexAdapter,
    msg: DexInstantiateMsg,
) -> DexResult {
    let recipient = adapter
        .account_registry(deps.as_ref())?
        .proxy_address(&AccountId::new(msg.recipient_account, AccountTrace::Local)?)?;
    let fee = UsageFee::new(deps.api, msg.swap_fee, recipient)?;
    SWAP_FEE.save(deps.storage, &fee)?;
    Ok(Response::default())
}
