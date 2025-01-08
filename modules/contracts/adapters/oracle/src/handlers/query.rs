use abstract_adapter::sdk::features::AbstractNameService;

use abstract_oracle_standard::msg::OracleQueryMsg;
use cosmwasm_std::{to_json_binary, Binary, Deps, Env};

use crate::contract::{OracleAdapter, OracleResult};
use crate::oracles;

pub fn query_handler(
    deps: Deps,
    env: Env,
    module: &OracleAdapter,
    msg: OracleQueryMsg,
) -> OracleResult<Binary> {
    match msg {
        OracleQueryMsg::Price {
            price_source_key,
            oracle,
            max_age,
        } => {
            let oracle = oracles::resolve_oracle(&oracle)?;
            let ans = module.name_service(deps);
            let price_response = oracle.price(deps, &env, ans.host(), price_source_key, max_age)?;

            to_json_binary(&price_response).map_err(Into::into)
        }
    }
}
