use crate::contract::{AppResult, ChallengeApp};
use crate::msg::{ChallengeQueryMsg, ChallengeResponse, ConfigResponse, FriendResponse};
use crate::state::{CHALLENGE_FRIENDS, CHALLENGE_LIST, CONFIG};
use abstract_sdk::features::AbstractNameService;
use cosmwasm_std::{to_binary, Binary, Deps, Env};
use cw_asset::AssetInfo;

pub fn query_handler(
    deps: Deps,
    _env: Env,
    app: &ChallengeApp,
    msg: ChallengeQueryMsg,
) -> AppResult<Binary> {
    match msg {
        ChallengeQueryMsg::Config {} => to_binary(&query_config(deps, app)?),
        ChallengeQueryMsg::Challenge { challenge_id } => {
            to_binary(&query_challenge(deps, app, challenge_id)?)
        }
        ChallengeQueryMsg::Friend {
            challenge_id,
            friend_address,
        } => to_binary(&query_friend(deps, app, challenge_id, friend_address)?),
    }
    .map_err(Into::into)
}

fn query_config(deps: Deps, app: &ChallengeApp) -> AppResult<ConfigResponse> {
    let config = CONFIG.load(deps.storage)?;
    let asset = AssetInfo::native(config.native_denom);
    let native_asset = app
        .ans_host(deps)?
        .query_asset_reverse(&deps.querier, &asset)?;
    Ok(ConfigResponse {
        native_asset,
        forfeit_amount: config.forfeit_amount,
    })
}

fn query_challenge(
    deps: Deps,
    _app: &ChallengeApp,
    challenge_id: String,
) -> AppResult<ChallengeResponse> {
    let challenge = CHALLENGE_LIST.may_load(deps.storage, challenge_id)?;
    Ok(ChallengeResponse { challenge })
}

fn query_friend(
    deps: Deps,
    _app: &ChallengeApp,
    challenge_id: String,
    friend_id: String,
) -> AppResult<FriendResponse> {
    let friend = CHALLENGE_FRIENDS.may_load(deps.storage, (friend_id, challenge_id))?;
    Ok(FriendResponse { friend })
}
