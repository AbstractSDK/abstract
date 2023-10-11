use crate::contract::{BetApp, BetResult};
use crate::error::BetError;
use crate::msg::{
    BetQueryMsg, BetsResponse, ConfigResponse, ListOddsResponse, OddsResponse, RoundResponse,
    RoundsResponse,
};
use crate::state::{AccountOdds, Config, Round, RoundId, RoundInfo, BETS, CONFIG, ODDS, ROUNDS};
use abstract_core::objects::AccountId;
use cosmwasm_std::{to_binary, Binary, Decimal, Deps, Env, Order, StdResult, Storage, Uint128};
use cw_storage_plus::Bound;

pub fn query_handler(deps: Deps, _env: Env, _etf: &BetApp, msg: BetQueryMsg) -> BetResult<Binary> {
    match msg {
        BetQueryMsg::Config {} => {
            let Config { rake } = CONFIG.load(deps.storage)?;
            to_binary(&ConfigResponse { rake: rake.share() })
        }
        BetQueryMsg::ListRounds { limit, start_after } => {
            to_binary(&list_rounds(deps, limit, start_after)?)
        }
        BetQueryMsg::Round { round_id } => {
            let round_res = Round::new(round_id).query(deps)?;
            to_binary(&round_res)
        }
        BetQueryMsg::Odds { round_id, team_id } => {
            let odds = ODDS.load(deps.storage, (round_id, team_id))?;
            to_binary(&OddsResponse { round_id, odds })
        }
        BetQueryMsg::ListOdds { round_id } => to_binary(&list_odds(deps, round_id)?),
        BetQueryMsg::Bets { round_id } => {
            let round = Round::new(round_id.clone());
            let bets = round.bets(deps.storage)?;
            let response = BetsResponse { round_id, bets };
            to_binary(&response)
        }
    }
    .map_err(Into::into)
}

fn list_rounds(
    deps: Deps,
    limit: Option<u32>,
    start_after: Option<RoundId>,
) -> BetResult<RoundsResponse> {
    let limit = limit.unwrap_or(10) as usize;

    let rounds: Vec<(RoundId, RoundInfo)> = ROUNDS
        .range(
            deps.storage,
            start_after.map(Bound::exclusive),
            None,
            Order::Ascending,
        )
        .take(limit)
        .collect::<StdResult<Vec<_>>>()?;

    let mut rounds_res = vec![];

    for (id, info) in rounds {
        let round_res = Round::new(id).query(deps)?;
        rounds_res.push(round_res);
    }

    Ok(RoundsResponse { rounds: rounds_res })
}

fn list_odds(deps: Deps, round_id: RoundId) -> BetResult<ListOddsResponse> {
    let odds = ODDS
        .prefix(round_id)
        .range(deps.storage, None, None, Order::Ascending)
        .collect::<StdResult<Vec<_>>>()?;
    let odds = odds
        .into_iter()
        .map(|(key, value)| AccountOdds {
            account_id: key,
            odds: value,
        })
        .collect::<Vec<AccountOdds>>();
    let response = ListOddsResponse { round_id, odds };
    Ok(response)
}

/// Returns the total bet amount for a specific `AccountId` in a given `RoundId`.
pub fn get_total_bets_for_team(
    storage: &dyn Storage,
    round_id: RoundId,
    account_id: AccountId,
) -> StdResult<Uint128> {
    let bets_for_account = BETS
        .may_load(storage, (round_id, account_id.clone()))?
        .unwrap_or_default();
    let total: Uint128 = bets_for_account.iter().map(|(_, amount)| *amount).sum();

    Ok(total)
}

/// Returns the total bet amount across all `AccountId`s for a given `RoundId`.
pub fn get_total_bets_for_all_accounts(
    storage: &dyn Storage,
    round_id: RoundId,
) -> StdResult<Uint128> {
    let all_keys = BETS
        .prefix(round_id)
        .keys(storage, None, None, Order::Ascending);

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
