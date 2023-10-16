use crate::contract::{AppResult, ChallengeApp};
use crate::msg::{
    ChallengeEntryResponse, ChallengeQueryMsg, ChallengeResponse, ChallengesResponse,
    FriendsResponse, PreviousProposalsResponse, VoteResponse, VotesResponse,
};
use crate::state::{CHALLENGES, CHALLENGE_FRIENDS, CHALLENGE_PROPOSALS, SIMPLE_VOTING};
use abstract_core::objects::voting::{ProposalId, ProposalInfo, VoteResult, DEFAULT_LIMIT};
use cosmwasm_std::{to_binary, Binary, Deps, Env, Order, StdResult};
use cw_storage_plus::Bound;

use super::execute::last_proposal;

pub fn query_handler(
    deps: Deps,
    env: Env,
    app: &ChallengeApp,
    msg: ChallengeQueryMsg,
) -> AppResult<Binary> {
    match msg {
        ChallengeQueryMsg::Challenge { challenge_id } => {
            to_binary(&query_challenge(deps, env, app, challenge_id)?)
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
            proposal_id,
        } => to_binary(&query_vote(
            deps,
            app,
            voter_addr,
            challenge_id,
            proposal_id,
        )?),
        ChallengeQueryMsg::PreviousProposals {
            challenge_id,
            start_after,
            limit,
        } => to_binary(&query_previous_proposal_results(
            deps,
            env,
            app,
            challenge_id,
            start_after,
            limit,
        )?),
        ChallengeQueryMsg::Votes {
            challenge_id,
            proposal_id,
            start_after,
            limit,
        } => to_binary(&query_votes(
            deps,
            app,
            challenge_id,
            proposal_id,
            start_after,
            limit,
        )?),
    }
    .map_err(Into::into)
}

fn query_challenge(
    deps: Deps,
    env: Env,
    _app: &ChallengeApp,
    challenge_id: u64,
) -> AppResult<ChallengeResponse> {
    let challenge = CHALLENGES.may_load(deps.storage, challenge_id)?;

    let challenge = if let Some(entry) = challenge {
        Some(ChallengeEntryResponse::from_entry(entry, challenge_id))
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

    let challenges = CHALLENGES
        .range(deps.storage, min, None, Order::Ascending)
        .take(limit as usize)
        .map(|result| {
            result
                // Map err into AppError
                .map_err(Into::into)
                // Cast result into response
                .and_then(|(challenge_id, entry)| {
                    Ok(ChallengeEntryResponse::from_entry(entry, challenge_id))
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
    proposal_id: Option<u64>,
) -> AppResult<VoteResponse> {
    let voter = deps.api.addr_validate(&voter_addr)?;
    let challenge = CHALLENGES.load(deps.storage, challenge_id)?;
    let maybe_proposal_id = if let Some(proposal_id) = proposal_id {
        // Only allow loading proposal_id for this challenge
        CHALLENGE_PROPOSALS
            .may_load(deps.storage, (challenge_id, proposal_id))?
            .map(|_| proposal_id)
    } else {
        last_proposal(challenge_id, deps)?
    };
    let vote = if let Some(proposal_id) = maybe_proposal_id {
        SIMPLE_VOTING.load_vote(deps.storage, proposal_id, &voter)?
    } else {
        None
    };
    Ok(VoteResponse { vote })
}

fn query_previous_proposal_results(
    deps: Deps,
    env: Env,
    _app: &ChallengeApp,
    challenge_id: u64,
    start_after: Option<ProposalId>,
    limit: Option<u64>,
) -> AppResult<PreviousProposalsResponse> {
    let min = start_after.map(Bound::exclusive);
    let limit = limit.unwrap_or(DEFAULT_LIMIT);
    let ids: Vec<ProposalId> = CHALLENGE_PROPOSALS
        .prefix(challenge_id)
        .keys(deps.storage, min, None, Order::Ascending)
        .take(limit as usize)
        .collect::<StdResult<_>>()?;
    let results = ids
        .into_iter()
        .map(|id| {
            SIMPLE_VOTING
                .load_proposal(deps.storage, &env.block, id)
                .map(|v| (id, v))
        })
        .collect::<VoteResult<Vec<(ProposalId, ProposalInfo)>>>()?;

    Ok(PreviousProposalsResponse { results })
}

fn query_votes(
    deps: Deps,
    _app: &ChallengeApp,
    challenge_id: u64,
    proposal_id: Option<u64>,
    start_after: Option<cosmwasm_std::Addr>,
    limit: Option<u64>,
) -> AppResult<VotesResponse> {
    let challenge = CHALLENGES.load(deps.storage, challenge_id)?;
    let maybe_proposal_id = if let Some(proposal_id) = proposal_id {
        // Only allow loading proposal_id for this challenge
        CHALLENGE_PROPOSALS
            .may_load(deps.storage, (challenge_id, proposal_id))?
            .map(|_| proposal_id)
    } else {
        last_proposal(challenge_id, deps)?
    };
    let votes = if let Some(proposal_id) = maybe_proposal_id {
        SIMPLE_VOTING.query_by_id(deps.storage, proposal_id, start_after.as_ref(), limit)?
    } else {
        vec![]
    };
    Ok(VotesResponse { votes })
}
