use crate::contract::{AppResult, ChallengeApp};
use crate::msg::{
    ChallengeEntryResponse, ChallengeQueryMsg, ChallengeResponse, ChallengesResponse,
    FriendsResponse, PreviousProposalsResponse, VoteResponse, VotesResponse,
};
use crate::state::{CHALLENGE_FRIENDS, CHALLENGE_LIST, SIMPLE_VOTING};
use abstract_core::objects::voting::{ProposalInfo, VoteResult, DEFAULT_LIMIT};
use cosmwasm_std::{to_binary, Binary, Deps, Env, Order, StdError};
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
            previous_proposal_index,
        } => to_binary(&query_vote(
            deps,
            app,
            voter_addr,
            challenge_id,
            previous_proposal_index,
        )?),
        ChallengeQueryMsg::PreviousProposals { challenge_id } => {
            to_binary(&query_previous_proposal_results(deps, app, challenge_id)?)
        }
        ChallengeQueryMsg::Votes {
            challenge_id,
            previous_proposal_index,
            start_after,
            limit,
        } => to_binary(&query_votes(
            deps,
            app,
            challenge_id,
            previous_proposal_index,
            start_after,
            limit,
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

    let challenge = if let Some(entry) = challenge {
        let proposal_info = SIMPLE_VOTING.load_proposal(deps.storage, entry.current_proposal_id)?;
        Some(ChallengeEntryResponse::from_entry_and_proposal_info(
            entry,
            challenge_id,
            proposal_info,
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

    let challenges = CHALLENGE_LIST
        .range(deps.storage, min, None, Order::Ascending)
        .take(limit as usize)
        .map(|result| {
            result
                // Map err into AppError
                .map_err(Into::into)
                // Cast result into response
                .and_then(|(challenge_id, entry)| {
                    let proposal_info =
                        SIMPLE_VOTING.load_proposal(deps.storage, entry.current_proposal_id)?;
                    Ok(ChallengeEntryResponse::from_entry_and_proposal_info(
                        entry,
                        challenge_id,
                        proposal_info,
                    ))
                })
        })
        .collect::<AppResult<Vec<ChallengeEntryResponse>>>()?;
    Ok(ChallengesResponse { challenges })
}

fn query_friends(deps: Deps, _app: &ChallengeApp, challenge_id: u64) -> AppResult<FriendsResponse> {
    let friends = CHALLENGE_FRIENDS.may_load(deps.storage, challenge_id)?;
    Ok(FriendsResponse {
        friends: friends.unwrap_or_default(),
    })
}

fn query_vote(
    deps: Deps,
    _app: &ChallengeApp,
    voter_addr: String,
    challenge_id: u64,
    previous_proposal_index: Option<u64>,
) -> AppResult<VoteResponse> {
    let voter = deps.api.addr_validate(&voter_addr)?;
    let challenge = CHALLENGE_LIST.load(deps.storage, challenge_id)?;
    let proposal_id = if let Some(index) = previous_proposal_index {
        *challenge
            .previous_proposal_ids
            .get(index as usize)
            .ok_or(StdError::not_found(format!(
                "previous_proposal with index {index}"
            )))?
    } else {
        challenge.current_proposal_id
    };
    let vote = SIMPLE_VOTING.load_vote(deps.storage, proposal_id, &voter)?;
    Ok(VoteResponse { vote })
}

fn query_previous_proposal_results(
    deps: Deps,
    _app: &ChallengeApp,
    challenge_id: u64,
) -> AppResult<PreviousProposalsResponse> {
    let challenge = CHALLENGE_LIST.load(deps.storage, challenge_id)?;
    let results = challenge
        .previous_proposal_ids
        .iter()
        .map(|&id| SIMPLE_VOTING.load_proposal(deps.storage, id))
        .collect::<VoteResult<Vec<ProposalInfo>>>()?;
    Ok(PreviousProposalsResponse { results })
}

fn query_votes(
    deps: Deps,
    _app: &ChallengeApp,
    challenge_id: u64,
    previous_proposal_index: Option<u64>,
    start_after: Option<cosmwasm_std::Addr>,
    limit: Option<u64>,
) -> AppResult<VotesResponse> {
    let challenge = CHALLENGE_LIST.load(deps.storage, challenge_id)?;
    let proposal_id = if let Some(index) = previous_proposal_index {
        *challenge
            .previous_proposal_ids
            .get(index as usize)
            .ok_or(StdError::not_found(format!(
                "previous_proposal with index {index}"
            )))?
    } else {
        challenge.current_proposal_id
    };
    let votes =
        SIMPLE_VOTING.query_by_id(deps.storage, proposal_id, start_after.as_ref(), limit)?;
    Ok(VotesResponse { votes })
}
