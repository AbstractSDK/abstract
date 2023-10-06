use crate::contract::{BetApp, BetResult};
use crate::msg::{BetQueryMsg, ConfigResponse};
use crate::state::{BETS, CONFIG, Config, TrackId};
use cosmwasm_std::{Binary, Deps, Env, Order, StdResult, Storage, to_binary, Uint128};
use abstract_core::objects::AccountId;

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


/// Returns the total bet amount for a specific `AccountId` in a given `TrackId`.
pub fn get_total_bets_for_account(
    storage: &dyn Storage,
    track_id: TrackId,
    account_id: AccountId
) -> StdResult<Uint128> {
    let bets_for_account = BETS.may_load(storage, (track_id, account_id))?.unwrap_or_default();
    let total: Uint128 = bets_for_account.iter().map(|(_, amount)| *amount).sum();
    Ok(total)
}

/// Returns the total bet amount across all `AccountId`s for a given `TrackId`.
pub fn get_total_bets_for_all_accounts(
    storage: &dyn Storage,
    track_id: TrackId
) -> StdResult<Uint128> {
    let all_keys = BETS.prefix(track_id).keys(storage, None, None, Order::Ascending);

    let total = all_keys
        .into_iter()
        .filter_map(|key| {
            let key = (track_id, key.ok()?);
            BETS.load(storage, key).ok()
        })
        .flatten()
        .map(|(_, amount)| amount)
        .sum();

    Ok(total)
}
