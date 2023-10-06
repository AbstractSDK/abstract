use crate::contract::{BetApp, BetResult};
use crate::msg::{BetQueryMsg, ConfigResponse, RoundResponse, RoundsResponse};
use crate::state::{BETS, CONFIG, Config, Round, RoundId, RoundInfo, ROUNDS};
use cosmwasm_std::{Binary, Deps, Env, Order, StdResult, Storage, to_binary, Uint128};
use abstract_core::objects::AccountId;
use cw_storage_plus::Bound;

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
        BetQueryMsg::Rounds { limit, start_after } => {
            let limit = limit.unwrap_or(10) as usize;

            let rounds: Vec<(RoundId, RoundInfo)> = ROUNDS
                .range(deps.storage, start_after.map(Bound::exclusive), None, Order::Ascending)
                .take(limit)
                .collect::<StdResult<Vec<_>>>()?;

            let mut rounds_res = vec![];

            for (id, info) in rounds {
                let round_res = Round::new(id).query(deps)?;
                rounds_res.push(round_res);
            }


            to_binary(&RoundsResponse { rounds: rounds_res })
        }
        BetQueryMsg::Round { round_id } => {
            let round_res = Round::new(round_id).query(deps)?;
            to_binary(&round_res)
        }
        _ => panic!("Unsupported query message"),
    }
    .map_err(Into::into)
}


/// Returns the total bet amount for a specific `AccountId` in a given `RoundId`.
pub fn get_total_bets_for_account(
    storage: &dyn Storage,
    round_id: RoundId,
    account_id: AccountId
) -> StdResult<Uint128> {
    let bets_for_account = BETS.may_load(storage, (round_id, account_id))?.unwrap_or_default();
    let total: Uint128 = bets_for_account.iter().map(|(_, amount)| *amount).sum();
    Ok(total)
}

/// Returns the total bet amount across all `AccountId`s for a given `RoundId`.
pub fn get_total_bets_for_all_accounts(
    storage: &dyn Storage,
    round_id: RoundId
) -> StdResult<Uint128> {
    let all_keys = BETS.prefix(round_id).keys(storage, None, None, Order::Ascending);

    let total = all_keys
        .into_iter()
        .filter_map(|key| {
            let key = (round_id, key.ok()?);
            BETS.load(storage, key).ok()
        })
        .flatten()
        .map(|(_, amount)| amount)
        .sum();

    Ok(total)
}
