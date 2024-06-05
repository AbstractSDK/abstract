pub const INSTANTIATE_REPLY_ID: u64 = 1u64;

use crate::contract::MyStandaloneResult;

use abstract_standalone::sdk::AbstractResponse;
use cosmwasm_std::{DepsMut, Env, Reply};

#[cfg_attr(feature = "export", cosmwasm_std::entry_point)]
pub fn reply(_deps: DepsMut, _env: Env, msg: Reply) -> MyStandaloneResult {
    match msg.id {
        self::INSTANTIATE_REPLY_ID => Ok(crate::MY_STANDALONE.response("instantiate_reply")),
        _ => todo!(),
    }
}
