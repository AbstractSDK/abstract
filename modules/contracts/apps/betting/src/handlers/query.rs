use crate::contract::{EtfApp, EtfResult};
use crate::msg::{BetQueryMsg, ConfigResponse};
use crate::state::{CONFIG, Config, STATE};
use cosmwasm_std::{to_binary, Binary, Deps, Env};

pub fn query_handler(deps: Deps, _env: Env, _etf: &EtfApp, msg: BetQueryMsg) -> EtfResult<Binary> {
    match msg {
        BetQueryMsg::Config {} => {
            let Config {
                rake
            } = CONFIG.load(deps.storage)?;
            to_binary(&ConfigResponse {
                rake: rake.share(),
            })
        }
        _ => panic!("Unsupported query message"),
    }
    .map_err(Into::into)
}
