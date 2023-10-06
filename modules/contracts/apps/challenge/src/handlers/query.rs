use crate::contract::{AppResult, ChallengeApp};
use crate::msg::{
    ChallengeEntryResponse, ChallengeQueryMsg, ChallengeResponse, ChallengesResponse,
    FriendsResponse, VoteResponse,
};
use crate::state::{CHALLENGE_FRIENDS, CHALLENGE_LIST, SIMPLE_VOTING};
use abstract_core::objects::voting::DEFAULT_LIMIT;
use cosmwasm_std::{to_binary, Binary, Deps, Env, Order};
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

    let challenge = if let Some(entry) = challenge {
        let vote_info = SIMPLE_VOTING.load_vote_info(deps.storage, entry.current_vote_id)?;
        Some(ChallengeEntryResponse::from_entry_and_vote_info(
            entry, vote_info,
        ))
    } else {
        None
    };
    Ok(ChallengeResponse { challenge })
}

fn query_challenges(
    deps: Deps,
    start: Option<u64>,
    limit: Option<u64>,
) -> AppResult<ChallengesResponse> {
    let min = start.map(Bound::exclusive);
    let limit = limit.unwrap_or(DEFAULT_LIMIT);
    let mut last_challenge_id = 0;

    let challenges = CHALLENGE_LIST
        .range(deps.storage, min, None, Order::Ascending)
        .take(limit as usize)
        .map(|result| {
            result
                // Map err into AppError
                .map_err(Into::into)
                // Cast result into response
                .and_then(|(challenge_id, entry)| {
                    last_challenge_id = challenge_id;
                    let vote_info =
                        SIMPLE_VOTING.load_vote_info(deps.storage, entry.current_vote_id)?;
                    Ok(ChallengeEntryResponse::from_entry_and_vote_info(
                        entry, vote_info,
                    ))
                })
        })
        .collect::<AppResult<Vec<ChallengeEntryResponse>>>()?;
    Ok(ChallengesResponse {
        challenges,
        last_index: last_challenge_id,
    })
}

fn query_friends(deps: Deps, _app: &ChallengeApp, challenge_id: u64) -> AppResult<FriendsResponse> {
    let friends = CHALLENGE_FRIENDS.may_load(deps.storage, challenge_id)?;
    Ok(FriendsResponse {
        friends: friends.map(Vec::from_iter).unwrap_or_default(),
    })
}

fn query_vote(
    deps: Deps,
    _app: &ChallengeApp,
    voter_addr: String,
    challenge_id: u64,
) -> AppResult<VoteResponse> {
    let voter = deps.api.addr_validate(&voter_addr)?;
    let challenge = CHALLENGE_LIST.load(deps.storage, challenge_id)?;
    let vote = SIMPLE_VOTING.load_vote(deps.storage, challenge.current_vote_id, &voter)?;
    Ok(VoteResponse { vote })
}
