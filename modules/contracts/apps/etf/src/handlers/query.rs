use cosmwasm_std::{to_json_binary, Binary, Deps, Env};

use crate::{
    contract::{EtfApp, EtfResult},
    msg::{EtfQueryMsg, StateResponse},
    state::{State, FEE, STATE},
};

pub fn query_handler(deps: Deps, _env: Env, _etf: &EtfApp, msg: EtfQueryMsg) -> EtfResult<Binary> {
    match msg {
        EtfQueryMsg::State {} => {
            let State {
                share_token_address,
                manager_addr,
            } = STATE.load(deps.storage)?;
            let fee = FEE.load(deps.storage)?;
            to_json_binary(&StateResponse {
                share_token_address,
                manager_addr,
                fee: fee.share(),
            })
        }
    }
    .map_err(Into::into)
}
