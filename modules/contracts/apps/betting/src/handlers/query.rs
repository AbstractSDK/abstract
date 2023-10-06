use crate::contract::{BetApp, BetResult};
use crate::msg::{BetQueryMsg, ConfigResponse};
use crate::state::{CONFIG, Config, STATE};
use cosmwasm_std::{to_binary, Binary, Deps, Env};

pub fn query_handler(deps: Deps, _env: Env, _etf: &BetApp, msg: BetQueryMsg) -> BetResult<Binary> {
    match msg {
        BetQueryMsg::Config {} => {
            let Config {
                rake,
                bet_asset
            } = CONFIG.load(deps.storage)?;
            to_binary(&ConfigResponse {
                rake: rake.share(),
                bet_asset,
            })
        }
        _ => panic!("Unsupported query message"),
    }
    .map_err(Into::into)
}
