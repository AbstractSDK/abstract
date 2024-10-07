use abstract_adapter::sdk::AccountVerification;
use abstract_adapter::std::objects::{account::AccountTrace, AccountId};
use abstract_dex_standard::msg::{DexFees, DexInstantiateMsg};
use cosmwasm_std::{DepsMut, Env, MessageInfo, Response};

use crate::{
    contract::{DexAdapter, DexResult},
    state::DEX_FEES,
};

pub fn instantiate_handler(
    deps: DepsMut,
    env: Env,
    _info: MessageInfo,
    module: DexAdapter,
    msg: DexInstantiateMsg,
) -> DexResult {
    let recipient = module
        .account_registry(deps.as_ref(), &env)?
        .account(&AccountId::new(msg.recipient_account, AccountTrace::Local)?)?;
    let dex_fees = DexFees::new(msg.swap_fee, recipient.into_addr())?;
    DEX_FEES.save(deps.storage, &dex_fees)?;
    Ok(Response::default())
}
