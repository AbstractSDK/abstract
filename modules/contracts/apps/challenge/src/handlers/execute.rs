use std::collections::HashSet;

use abstract_app::{
    sdk::{
        features::AbstractResponse, AbstractSdkResult, AccountVerification, Execution,
        TransferInterface,
    },
    std::objects::{
        voting::{ProposalId, ProposalInfo, ProposalOutcome, Vote},
        AnsAsset,
    },
};
use cosmwasm_std::{
    ensure, Addr, Deps, DepsMut, Empty, Env, MessageInfo, Order, Response, StdResult, Uint128,
};

use crate::{
    contract::{AppResult, ChallengeApp},
    error::AppError,
    msg::{ChallengeExecuteMsg, ChallengeRequest, Friend},
    state::{
        ChallengeEntry, ChallengeEntryUpdate, UpdateFriendsOpKind, CHALLENGES, CHALLENGE_FRIENDS,
        CHALLENGE_PROPOSALS, MAX_AMOUNT_OF_FRIENDS, NEXT_ID, SIMPLE_VOTING,
    },
};

pub fn execute_handler(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
    module: ChallengeApp,
    msg: ChallengeExecuteMsg,
) -> AppResult {
    match msg {
        ChallengeExecuteMsg::CreateChallenge { challenge_req } => {
            create_challenge(deps, env, info, module, challenge_req)
        }
        ChallengeExecuteMsg::UpdateChallenge {
            challenge_id,
            challenge,
        } => update_challenge(deps, env, info, module, challenge_id, challenge),
        ChallengeExecuteMsg::CancelChallenge { challenge_id } => {
            cancel_challenge(deps, env, info, &module, challenge_id)
        }
        ChallengeExecuteMsg::UpdateFriendsForChallenge {
            challenge_id,
            friends,
            op_kind,
        } => update_friends_for_challenge(deps, env, info, &module, challenge_id, friends, op_kind),
        ChallengeExecuteMsg::CastVote {
            vote_to_punish: vote,
            challenge_id,
        } => cast_vote(deps, env, info, &module, vote, challenge_id),
        ChallengeExecuteMsg::CountVotes { challenge_id } => {
            count_votes(deps, env, info, &module, challenge_id)
        }
        ChallengeExecuteMsg::Veto { challenge_id } => veto(deps, env, info, &module, challenge_id),
        ChallengeExecuteMsg::UpdateConfig { new_vote_config } => {
            SIMPLE_VOTING.update_vote_config(deps.storage, &new_vote_config)?;
            Ok(Response::new())
        }
    }
}

/// Create new challenge
fn create_challenge(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
    module: ChallengeApp,
    challenge_req: ChallengeRequest,
) -> AppResult {
    // Only the admin should be able to create a challenge.
    module
        .admin
        .assert_admin(deps.as_ref(), &env, &info.sender)?;
    ensure!(
        challenge_req.init_friends.len() < MAX_AMOUNT_OF_FRIENDS as usize,
        AppError::TooManyFriends {}
    );
    // Validate friend addr and account ids
    let friends_validated: Vec<(Addr, Friend<Addr>)> = challenge_req
        .init_friends
        .iter()
        .cloned()
        .map(|human| human.check(deps.as_ref(), &module))
        .collect::<AbstractSdkResult<_>>()?;

    let (friend_addrs, friends): (Vec<Addr>, Vec<Friend<Addr>>) =
        friends_validated.into_iter().unzip();
    // Check if addrs unique
    let mut unique_addrs = HashSet::with_capacity(friend_addrs.len());
    if !friend_addrs.iter().all(|x| unique_addrs.insert(x)) {
        return Err(AppError::DuplicateFriends {});
    }

    // Generate the challenge id
    let challenge_id = NEXT_ID.update(deps.storage, |id| AppResult::Ok(id + 1))?;
    CHALLENGE_FRIENDS.save(deps.storage, challenge_id, &friends)?;

    // Create new challenge
    let end_timestamp = env
        .block
        .time
        .plus_seconds(challenge_req.challenge_duration_seconds.u64());
    let challenge = ChallengeEntry::new(challenge_req, end_timestamp)?;
    CHALLENGES.save(deps.storage, challenge_id, &challenge)?;

    Ok(module
        .response("create_challenge")
        .add_attribute("challenge_id", challenge_id.to_string()))
}

fn update_challenge(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
    module: ChallengeApp,
    challenge_id: u64,
    new_challenge: ChallengeEntryUpdate,
) -> AppResult {
    module
        .admin
        .assert_admin(deps.as_ref(), &env, &info.sender)?;

    // will return an error if the challenge doesn't exist
    let mut loaded_challenge: ChallengeEntry = CHALLENGES
        .may_load(deps.storage, challenge_id)?
        .ok_or(AppError::ChallengeNotFound {})?;

    if let Some(name) = new_challenge.name {
        loaded_challenge.name = name;
    }

    if let Some(description) = new_challenge.description {
        loaded_challenge.description = description;
    }

    // Save the updated challenge
    CHALLENGES.save(deps.storage, challenge_id, &loaded_challenge)?;

    Ok(module
        .response("update_challenge")
        .add_attribute("challenge_id", challenge_id.to_string()))
}

fn cancel_challenge(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
    module: &ChallengeApp,
    challenge_id: u64,
) -> AppResult {
    module
        .admin
        .assert_admin(deps.as_ref(), &env, &info.sender)?;
    let mut challenge = CHALLENGES.load(deps.storage, challenge_id)?;
    // Check if this challenge still active
    if env.block.time >= challenge.end_timestamp {
        return Err(AppError::ChallengeExpired {});
    }

    // If there is active proposal - cancel it
    let last_proposal_id = last_proposal(challenge_id, deps.as_ref())?;
    if let Some(proposal_id) = last_proposal_id {
        SIMPLE_VOTING.cancel_proposal(deps.storage, &env.block, proposal_id)?;
    }

    // End it now
    challenge.end_timestamp = env.block.time;
    CHALLENGES.save(deps.storage, challenge_id, &challenge)?;

    Ok(module
        .response("cancel_challenge")
        .add_attribute("challenge_id", challenge_id.to_string()))
}

fn update_friends_for_challenge(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
    module: &ChallengeApp,
    challenge_id: u64,
    friends: Vec<Friend<String>>,
    op_kind: UpdateFriendsOpKind,
) -> AppResult {
    module
        .admin
        .assert_admin(deps.as_ref(), &env, &info.sender)?;
    // Validate friend addr and account ids
    let friends_validated: Vec<(Addr, Friend<Addr>)> = friends
        .iter()
        .cloned()
        .map(|human| human.check(deps.as_ref(), module))
        .collect::<AbstractSdkResult<_>>()?;

    let (voters_addrs, friends): (Vec<Addr>, Vec<Friend<Addr>>) =
        friends_validated.into_iter().unzip();

    let last_proposal_id = last_proposal(challenge_id, deps.as_ref())?;

    // Don't allow edit friends if last proposal haven't ended yet
    if let Some(proposal_id) = last_proposal_id {
        let info = SIMPLE_VOTING.load_proposal(deps.storage, &env.block, proposal_id)?;
        if env.block.time < info.end_timestamp {
            return Err(AppError::FriendsEditDuringProposal(info.end_timestamp));
        }
    }

    match op_kind {
        UpdateFriendsOpKind::Add {} => {
            let mut current_friends = CHALLENGE_FRIENDS
                .may_load(deps.storage, challenge_id)?
                .ok_or(AppError::ChallengeNotFound {})?;

            ensure!(
                friends.len() + current_friends.len() < MAX_AMOUNT_OF_FRIENDS as usize,
                AppError::TooManyFriends {}
            );

            let mut current_friends_addrs: Vec<Addr> = current_friends
                .iter()
                .map(|f| f.addr(deps.as_ref(), module))
                .collect::<AbstractSdkResult<_>>()?;
            current_friends_addrs.extend(voters_addrs);
            // Check if addrs unique
            let mut unique_addrs = HashSet::with_capacity(current_friends_addrs.len());
            if !current_friends_addrs.iter().all(|x| unique_addrs.insert(x)) {
                return Err(AppError::DuplicateFriends {});
            }

            current_friends.extend(friends);
            CHALLENGE_FRIENDS.save(deps.storage, challenge_id, &current_friends)?;
        }
        UpdateFriendsOpKind::Remove {} => {
            CHALLENGE_FRIENDS.update(deps.storage, challenge_id, |current_friends| {
                let mut current_friends = current_friends.ok_or(AppError::ZeroFriends {})?;
                for rem_friend in friends.iter() {
                    current_friends.retain(|friend| friend != rem_friend);
                }
                AppResult::Ok(current_friends)
            })?;
        }
    }
    Ok(module
        .response("update_friends")
        .add_attribute("challenge_id", challenge_id.to_string()))
}

fn get_or_create_active_proposal(
    deps: &mut DepsMut,
    env: &Env,
    challenge_id: u64,
    module: &ChallengeApp,
) -> AppResult<ProposalId> {
    let challenge = CHALLENGES.load(deps.storage, challenge_id)?;

    // Load last proposal and use it if it's active
    if let Some(proposal_id) = last_proposal(challenge_id, deps.as_ref())? {
        let proposal = SIMPLE_VOTING.load_proposal(deps.storage, &env.block, proposal_id)?;
        if proposal.assert_active_proposal().is_ok() {
            return Ok(proposal_id);
        }
    }

    // Or create a new one otherwise
    if env.block.time >= challenge.end_timestamp {
        return Err(AppError::ChallengeExpired {});
    }
    let friends: Vec<Addr> = CHALLENGE_FRIENDS
        .load(deps.storage, challenge_id)?
        .into_iter()
        .map(|friend| friend.addr(deps.as_ref(), module))
        .collect::<AbstractSdkResult<_>>()?;
    let proposal_id = SIMPLE_VOTING.new_proposal(
        deps.storage,
        env.block
            .time
            .plus_seconds(challenge.proposal_duration_seconds.u64()),
        &friends,
    )?;
    CHALLENGE_PROPOSALS.save(deps.storage, (challenge_id, proposal_id), &Empty {})?;

    Ok(proposal_id)
}

fn cast_vote(
    mut deps: DepsMut,
    env: Env,
    info: MessageInfo,
    module: &ChallengeApp,
    vote: Vote,
    challenge_id: u64,
) -> AppResult {
    let proposal_id = get_or_create_active_proposal(&mut deps, &env, challenge_id, module)?;

    let voter = match module
        .account_registry(deps.as_ref())?
        .assert_is_account(&info.sender)
    {
        Ok(base) => base.into_addr(),
        Err(_) => info.sender,
    };
    let proposal_info =
        SIMPLE_VOTING.cast_vote(deps.storage, &env.block, proposal_id, &voter, vote)?;

    Ok(module
        .response("cast_vote")
        .add_attribute("proposal_info", format!("{proposal_info:?}")))
}

fn count_votes(
    deps: DepsMut,
    env: Env,
    _info: MessageInfo,
    module: &ChallengeApp,
    challenge_id: u64,
) -> AppResult {
    let challenge = CHALLENGES.load(deps.storage, challenge_id)?;
    let proposal_id =
        last_proposal(challenge_id, deps.as_ref())?.ok_or(AppError::ExpectedProposal {})?;
    let (proposal_info, outcome) =
        SIMPLE_VOTING.count_votes(deps.storage, &env.block, proposal_id)?;

    try_finish_challenge(
        deps,
        module,
        proposal_info,
        outcome,
        challenge,
        challenge_id,
    )
}

fn veto(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
    module: &ChallengeApp,
    challenge_id: u64,
) -> AppResult {
    let proposal_id =
        last_proposal(challenge_id, deps.as_ref())?.ok_or(AppError::ExpectedProposal {})?;

    module
        .admin
        .assert_admin(deps.as_ref(), &env, &info.sender)?;
    let proposal_info = SIMPLE_VOTING.veto_proposal(deps.storage, &env.block, proposal_id)?;

    Ok(module
        .response("veto")
        .add_attribute("proposal_info", format!("{proposal_info:?}")))
}

fn try_finish_challenge(
    deps: DepsMut,
    module: &ChallengeApp,
    proposal_info: ProposalInfo,
    proposal_outcome: ProposalOutcome,
    mut challenge: ChallengeEntry,
    challenge_id: u64,
) -> AppResult {
    let friends = CHALLENGE_FRIENDS.load(deps.storage, challenge_id)?;
    let challenge_finished = if matches!(proposal_outcome, ProposalOutcome::Passed) {
        challenge.admin_strikes.strike()
    } else {
        false
    };

    // Return here if not required to charge penalty
    let res = if !matches!(proposal_outcome, ProposalOutcome::Passed) {
        module.response("finish_vote")
    } else {
        charge_penalty(deps, module, challenge, friends)?
    };
    Ok(res
        .add_attribute("proposal_info", format!("{proposal_info:?}"))
        .add_attribute("challenge_finished", challenge_finished.to_string()))
}

fn charge_penalty(
    deps: DepsMut,
    module: &ChallengeApp,
    challenge: ChallengeEntry,
    friends: Vec<Friend<Addr>>,
) -> Result<Response, AppError> {
    let num_friends = friends.len() as u128;
    if num_friends == 0 {
        return Err(AppError::ZeroFriends {});
    }
    let (amount_per_friend, remainder) = match challenge.strike_strategy {
        crate::state::StrikeStrategy::Split(amount) => (
            Uint128::new(amount.u128() / num_friends),
            amount.u128() % num_friends,
        ),
        crate::state::StrikeStrategy::PerFriend(amount) => (amount, 0),
    };

    let asset_per_friend = AnsAsset {
        name: challenge.strike_asset,
        amount: amount_per_friend,
    };

    let bank = module.bank(deps.as_ref());
    let executor = module.executor(deps.as_ref());

    // Create a transfer action for each friend
    let transfer_actions = friends
        .into_iter()
        .map(|friend| {
            let recipent = match friend {
                Friend::Addr(addr) => addr.address,
                Friend::AbstractAccount(account_id) => module
                    .account_registry(deps.as_ref())?
                    .account(&account_id)?
                    .into_addr(),
            };
            bank.transfer(vec![asset_per_friend.clone()], &recipent)
        })
        .collect::<Result<Vec<_>, _>>()?;

    let transfer_msg = executor.execute(transfer_actions)?;

    Ok(module
        .response("charge_penalty")
        .add_message(transfer_msg)
        .add_attribute("remainder", remainder.to_string()))
}

pub(crate) fn last_proposal(challenge_id: u64, deps: Deps) -> StdResult<Option<ProposalId>> {
    CHALLENGE_PROPOSALS
        .prefix(challenge_id)
        .keys(deps.storage, None, None, Order::Descending)
        .next()
        .transpose()
}
