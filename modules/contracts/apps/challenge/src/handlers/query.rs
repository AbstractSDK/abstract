use crate::contract::{AppResult, ChallengeApp};
use crate::msg::{
    ChallengeQueryMsg, ChallengeResponse, ChallengesResponse, CheckInsResponse, FriendsResponse,
    VoteResponse,
};
use crate::state::{
    ChallengeEntry, Vote, CHALLENGE_FRIENDS, CHALLENGE_LIST, DAILY_CHECK_INS, VOTES,
};
use cosmwasm_std::{to_binary, Binary, Deps, Env, Order, StdResult};
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
        ChallengeQueryMsg::CheckIns { challenge_id } => {
            to_binary(&query_check_in(deps, app, challenge_id)?)
        }
        ChallengeQueryMsg::Vote {
            last_check_in,
            voter_addr,
            challenge_id,
        } => to_binary(&query_vote_for_check_in(
            deps,
            app,
            voter_addr,
            last_check_in,
            challenge_id,
        )?),
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
) -> AppResult<CheckInsResponse> {
    let check_ins = DAILY_CHECK_INS.may_load(deps.storage, challenge_id)?;
    Ok(CheckInsResponse(check_ins.unwrap_or_default()))
}

fn query_vote_for_check_in(
    deps: Deps,
    _app: &ChallengeApp,
    voter_addr: String,
    last_check_in: u64,
    challenge_id: u64,
) -> AppResult<VoteResponse> {
    let v = Vote {
        voter: voter_addr,
        approval: None,
        for_check_in: None,
    };
    let v = v.check(deps)?;
    let vote = VOTES.may_load(deps.storage, (challenge_id, last_check_in, v.voter))?;
    Ok(VoteResponse { vote })
}
