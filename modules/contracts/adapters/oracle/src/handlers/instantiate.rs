use abstract_core::objects::{account::AccountTrace, AccountId};
use abstract_dex_standard::msg::{DexFees, DexInstantiateMsg};
use abstract_sdk::AccountVerification;
use cosmwasm_std::{DepsMut, Env, MessageInfo, Response};

use crate::{
    contract::{OracleAdapter, OracleResult},
    state::DEX_FEES,
};

pub fn instantiate_handler(
    deps: DepsMut,
    _env: Env,
    _info: MessageInfo,
    adapter: OracleAdapter,
    msg: DexInstantiateMsg,
) -> OracleResult {
    let recipient = adapter
        .account_registry(deps.as_ref())?
        .proxy_address(&AccountId::new(msg.recipient_account, AccountTrace::Local)?)?;
    let dex_fees = DexFees::new(msg.swap_fee, recipient)?;
    DEX_FEES.save(deps.storage, &dex_fees)?;
    Ok(Response::default())
}
