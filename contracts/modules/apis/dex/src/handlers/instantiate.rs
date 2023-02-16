use crate::contract::{DexApi, DexResult};
use abstract_os::{
    dex::{state::SWAP_FEE, DexInstantiateMsg},
    objects::fee::UsageFee,
};
use abstract_sdk::OsVerification;
use cosmwasm_std::{DepsMut, Env, MessageInfo, Response};

pub fn instantiate_handler(
    deps: DepsMut,
    _env: Env,
    _info: MessageInfo,
    api: DexApi,
    msg: DexInstantiateMsg,
) -> DexResult {
    let recipient = api
        .os_registry(deps.as_ref())
        .proxy_address(msg.recipient_os)?;
    let fee = UsageFee::new(deps.api, msg.swap_fee, recipient)?;
    SWAP_FEE.save(deps.storage, &fee)?;
    Ok(Response::default())
}
