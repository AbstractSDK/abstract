use crate::contract::{AppResult, ChallengeApp};
use crate::msg::{
    ChallengeQueryMsg, ChallengeResponse, ChallengesResponse, FriendsResponse, VoteResponse,
};
use crate::state::{ChallengeEntry, CHALLENGE_LIST};
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
        ChallengeQueryMsg::Vote {
            voter_addr,
            challenge_id,
        } => to_binary(&query_vote_for_check_in(
            deps,
            app,
            voter_addr,
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
    todo!()
    // Ok(ChallengeResponse { challenge: challenge.map(f) })
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
    todo!()
    // let friends = CHALLENGE_FRIENDS.may_load(deps.storage, challenge_id)?;
    // Ok(FriendsResponse(friends.unwrap_or_default()))
}

fn query_vote_for_check_in(
    deps: Deps,
    _app: &ChallengeApp,
    voter_addr: String,
    challenge_id: u64,
) -> AppResult<VoteResponse> {
    todo!()
    // let v = Vote {
    //     voter: voter_addr,
    //     approval: None,
    // };
    // let v = v.check(deps)?;
    // let vote = VOTES.may_load(deps.storage, (challenge_id, v.voter))?;
    // Ok(VoteResponse { vote })
}
