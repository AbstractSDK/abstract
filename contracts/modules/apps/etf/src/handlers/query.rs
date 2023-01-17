use crate::contract::EtfApp;
use abstract_os::etf::state::{FEE, STATE};
use abstract_os::etf::{EtfQueryMsg, StateResponse};
use cosmwasm_std::{to_binary, Binary, Deps, Env, StdResult};

pub fn query_handler(deps: Deps, _env: Env, _etf: &EtfApp, msg: EtfQueryMsg) -> StdResult<Binary> {
    match msg {
        EtfQueryMsg::State {} => {
            let fee = FEE.load(deps.storage)?;
            to_binary(&StateResponse {
                liquidity_token: STATE.load(deps.storage)?.liquidity_token_addr.to_string(),
                fee: fee.share(),
            })
        }
    }
}
