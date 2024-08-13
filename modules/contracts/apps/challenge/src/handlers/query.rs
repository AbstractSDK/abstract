use abstract_app::std::objects::voting::{ProposalId, ProposalInfo, VoteResult, DEFAULT_LIMIT};
use cosmwasm_std::{to_json_binary, Binary, BlockInfo, Deps, Env, Order, StdResult};
use cw_storage_plus::Bound;

use super::execute::last_proposal;
use crate::{
    contract::{AppResult, ChallengeApp},
    msg::{
        ChallengeEntryResponse, ChallengeQueryMsg, ChallengeResponse, ChallengesResponse,
        FriendsResponse, ProposalsResponse, VoteResponse, VotesResponse,
    },
    state::{CHALLENGES, CHALLENGE_FRIENDS, CHALLENGE_PROPOSALS, SIMPLE_VOTING},
};

pub fn query_handler(
    deps: Deps,
    env: Env,
    module: &ChallengeApp,
    msg: ChallengeQueryMsg,
) -> AppResult<Binary> {
    match msg {
        ChallengeQueryMsg::Challenge { challenge_id } => {
            to_json_binary(&query_challenge(deps, env, module, challenge_id)?)
        }
        ChallengeQueryMsg::Challenges { start_after, limit } => {
            to_json_binary(&query_challenges(deps, env, start_after, limit)?)
        }
        ChallengeQueryMsg::Friends { challenge_id } => {
            to_json_binary(&query_friends(deps, module, challenge_id)?)
        }
        ChallengeQueryMsg::Vote {
            voter_addr,
            challenge_id,
            proposal_id,
        } => to_json_binary(&query_vote(
            deps,
            module,
            voter_addr,
            challenge_id,
            proposal_id,
        )?),
        ChallengeQueryMsg::Proposals {
            challenge_id,
            start_after,
            limit,
        } => to_json_binary(&query_proposals(
            deps,
            env,
            module,
            challenge_id,
            start_after,
            limit,
        )?),
        ChallengeQueryMsg::Votes {
            challenge_id,
            proposal_id,
            start_after,
            limit,
        } => to_json_binary(&query_votes(
            deps,
            module,
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
    _module: &ChallengeApp,
    challenge_id: u64,
) -> AppResult<ChallengeResponse> {
    let challenge = CHALLENGES.may_load(deps.storage, challenge_id)?;

    let proposal = get_proposal_if_active(challenge_id, deps, &env.block)?;
    let challenge =
        challenge.map(|entry| ChallengeEntryResponse::from_entry(entry, challenge_id, proposal));
    Ok(ChallengeResponse { challenge })
}

fn get_proposal_if_active(
    challenge_id: u64,
    deps: Deps,
    block: &BlockInfo,
) -> Result<Option<ProposalInfo>, crate::error::AppError> {
    let maybe_id = last_proposal(challenge_id, deps)?;
    let proposal = maybe_id
        .map(|id| {
            let proposal = SIMPLE_VOTING.load_proposal(deps.storage, block, id)?;
            if proposal.assert_active_proposal().is_ok() {
                AppResult::Ok(Some(proposal))
            } else {
                AppResult::Ok(None)
            }
        })
        .transpose()?
        .flatten();
    Ok(proposal)
}

fn query_challenges(
    deps: Deps,
    env: Env,
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
                .map_err(Into::into)
                // Cast result into response
                .map(|(challenge_id, entry)| {
                    let proposal =
                        get_proposal_if_active(challenge_id, deps, &env.block).unwrap_or_default();
                    ChallengeEntryResponse::from_entry(entry, challenge_id, proposal)
                })
        })
        .collect::<AppResult<Vec<ChallengeEntryResponse>>>()?;
    Ok(ChallengesResponse { challenges })
}

fn query_friends(
    deps: Deps,
    _module: &ChallengeApp,
    challenge_id: u64,
) -> AppResult<FriendsResponse> {
    let friends = CHALLENGE_FRIENDS.may_load(deps.storage, challenge_id)?;
    Ok(FriendsResponse {
        friends: friends.unwrap_or_default(),
    })
}

fn query_vote(
    deps: Deps,
    _module: &ChallengeApp,
    voter_addr: String,
    challenge_id: u64,
    proposal_id: Option<u64>,
) -> AppResult<VoteResponse> {
    let voter = deps.api.addr_validate(&voter_addr)?;
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

fn query_proposals(
    deps: Deps,
    env: Env,
    _moodule: &ChallengeApp,
    challenge_id: u64,
    start_after: Option<ProposalId>,
    limit: Option<u64>,
) -> AppResult<ProposalsResponse> {
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

    Ok(ProposalsResponse { proposals: results })
}

fn query_votes(
    deps: Deps,
    _module: &ChallengeApp,
    challenge_id: u64,
    proposal_id: Option<u64>,
    start_after: Option<cosmwasm_std::Addr>,
    limit: Option<u64>,
) -> AppResult<VotesResponse> {
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
