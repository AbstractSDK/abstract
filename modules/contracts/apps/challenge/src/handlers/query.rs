use crate::contract::{AppResult, ChallengeApp};
use crate::msg::{
    ChallengeQueryMsg, ChallengeResponse, CheckInResponse, FriendsResponse, VotesResponse,
};
use crate::state::{CHALLENGE_FRIENDS, CHALLENGE_LIST, DAILY_CHECK_INS, VOTES};
use cosmwasm_std::{to_binary, Binary, Deps, Env};

pub fn query_handler(
    deps: Deps,
    _env: Env,
    app: &ChallengeApp,
    msg: ChallengeQueryMsg,
) -> AppResult<Binary> {
    match msg {
        ChallengeQueryMsg::Challenge { challenge_id } => {
            to_binary(&query_challenge(deps, app, challenge_id)?)
        }
        ChallengeQueryMsg::Friends { challenge_id } => {
            to_binary(&query_friends(deps, app, challenge_id)?)
        }
        ChallengeQueryMsg::CheckIn { challenge_id } => {
            to_binary(&query_check_in(deps, app, challenge_id)?)
        }
        ChallengeQueryMsg::Votes { challenge_id } => {
            to_binary(&query_votes(deps, app, challenge_id)?)
        }
    }
    .map_err(Into::into)
}

fn query_challenge(
    deps: Deps,
    _app: &ChallengeApp,
    challenge_id: u64,
) -> AppResult<ChallengeResponse> {
    let challenge = CHALLENGE_LIST.may_load(deps.storage, challenge_id)?;
    Ok(ChallengeResponse { challenge })
}

fn query_friends(deps: Deps, _app: &ChallengeApp, challenge_id: u64) -> AppResult<FriendsResponse> {
    let friends = CHALLENGE_FRIENDS.may_load(deps.storage, challenge_id)?;
    Ok(FriendsResponse { friends })
}

fn query_check_in(
    deps: Deps,
    _app: &ChallengeApp,
    challenge_id: u64,
) -> AppResult<CheckInResponse> {
    let check_in = DAILY_CHECK_INS.may_load(deps.storage, challenge_id)?;
    Ok(CheckInResponse { check_in })
}

fn query_votes(deps: Deps, _app: &ChallengeApp, challenge_id: u64) -> AppResult<VotesResponse> {
    let votes = VOTES.may_load(deps.storage, challenge_id)?;
    Ok(VotesResponse { votes })
}
