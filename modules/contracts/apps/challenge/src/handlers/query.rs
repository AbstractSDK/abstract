use crate::contract::{AppResult, ChallengeApp};
use crate::msg::{
    ChallengeQueryMsg, ChallengeResponse, ChallengesResponse, CheckInResponse, FriendsResponse,
    VoteResponse,
};
use crate::state::{
    ChallengeEntry, Vote, CHALLENGE_FRIENDS, CHALLENGE_LIST, DAILY_CHECK_INS, VOTES,
};
use cosmwasm_std::{to_binary, Binary, Deps, Env, Order, StdResult, Timestamp};
use cw_storage_plus::Bound;

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
        ChallengeQueryMsg::Challenges { start_after, limit } => {
            to_binary(&query_challenges(deps, start_after, limit)?)
        }
        ChallengeQueryMsg::Friends { challenge_id } => {
            to_binary(&query_friends(deps, app, challenge_id)?)
        }
        ChallengeQueryMsg::CheckIn { challenge_id } => {
            to_binary(&query_check_in(deps, app, challenge_id)?)
        }
        ChallengeQueryMsg::Vote {
            challenge_id,
            voter_addr,
        } => to_binary(&query_vote(deps, app, voter_addr, challenge_id)?),
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

fn query_challenges(deps: Deps, start: u64, limit: u32) -> AppResult<ChallengesResponse> {
    let challenges: StdResult<Vec<ChallengeEntry>> = CHALLENGE_LIST
        .range(
            deps.storage,
            Some(Bound::exclusive(start)),
            Some(Bound::inclusive(limit)),
            Order::Ascending,
        )
        .map(|result| result.map(|(_, entry)| entry)) // strip the keys
        .collect::<StdResult<Vec<ChallengeEntry>>>();
    Ok(ChallengesResponse(challenges.unwrap_or_default()))
}

fn query_friends(deps: Deps, _app: &ChallengeApp, challenge_id: u64) -> AppResult<FriendsResponse> {
    let friends = CHALLENGE_FRIENDS.may_load(deps.storage, challenge_id)?;
    Ok(FriendsResponse(friends.unwrap_or_default()))
}

fn query_check_in(
    deps: Deps,
    _app: &ChallengeApp,
    challenge_id: u64,
) -> AppResult<CheckInResponse> {
    let check_in = DAILY_CHECK_INS.may_load(deps.storage, challenge_id)?;
    Ok(CheckInResponse { check_in })
}

fn query_vote(
    deps: Deps,
    _app: &ChallengeApp,
    voter_addr: String,
    challenge_id: u64,
) -> AppResult<VoteResponse> {
    let v = Vote {
        voter: voter_addr,
        approval: None,
    };
    let v = v.check(deps)?;
    let vote = VOTES.may_load(deps.storage, (challenge_id, v.voter))?;
    Ok(VoteResponse { vote })
}
