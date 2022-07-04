#[cfg(not(feature = "library"))]
use cosmwasm_std::entry_point;
use cosmwasm_std::{
    attr, from_binary, to_binary, Binary, CanonicalAddr, CosmosMsg, Decimal, Deps, DepsMut, Env,
    MessageInfo, Response, StdResult, Uint128, WasmMsg,
};
use cw20::{Cw20ExecuteMsg, Cw20ReceiveMsg};
use terraswap::querier::query_token_balance;

use crate::error::ContractError;
use crate::{ExecuteMsg, InstantiateMsg, QueryMsg};
use crate::staking::{query_staker, stake_voting_tokens, withdraw_voting_tokens};
use crate::state::{
    bank_read, bank_store, config_read, config_store, poll_indexer_store, poll_read, poll_store,
    poll_voter_read, poll_voter_store, read_poll_voters, read_polls, state_read, state_store,
    Config, ConfigResponse, Cw20HookMsg, ExecuteData, OrderBy, Poll, PollExecuteMsg, PollResponse,
    PollStatus, PollsResponse, State, StateResponse, VoteOption, VoterInfo, VotersResponse,
    VotersResponseItem,
};
use crate::validators::{
    validate_poll_description, validate_poll_link, validate_poll_title, validate_quorum,
    validate_threshold,
};

pub(crate) const MAX_QUORUM: Decimal = Decimal::one();
pub(crate) const MAX_THRESHOLD: Decimal = Decimal::one();
pub(crate) const MIN_TITLE_LENGTH: usize = 4;
pub(crate) const MAX_TITLE_LENGTH: usize = 64;
pub(crate) const MIN_DESC_LENGTH: usize = 4;
pub(crate) const MAX_DESC_LENGTH: usize = 1024;
pub(crate) const MIN_LINK_LENGTH: usize = 12;
pub(crate) const MAX_LINK_LENGTH: usize = 128;

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn instantiate(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
    msg: InstantiateMsg,
) -> Result<Response, ContractError> {
    validate_quorum(msg.quorum)?;
    validate_threshold(msg.threshold)?;

    let config = Config {
        whale_token: CanonicalAddr::from(vec![]),
        owner: deps.api.addr_canonicalize(info.sender.as_str())?,
        quorum: msg.quorum,
        threshold: msg.threshold,
        voting_period: msg.voting_period,
        timelock_period: msg.timelock_period,
        expiration_period: msg.expiration_period,
        proposal_deposit: msg.proposal_deposit,
        snapshot_period: msg.snapshot_period,
    };

    let state = State {
        contract_addr: deps.api.addr_canonicalize(env.contract.address.as_str())?,
        poll_count: 0,
        total_share: Uint128::zero(),
        total_deposit: Uint128::zero(),
    };
    config_store(deps.storage).save(&config)?;
    state_store(deps.storage).save(&state)?;

    Ok(Response::default())
}

// Routers; here is a separate router which handles Execution of functions on the contract or performs a contract Query
// Each router function defines a number of handlers using Rust's pattern matching to
// designated how each ExecutionMsg or QueryMsg will be handled.

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn execute(
    deps: DepsMut,
    _env: Env,
    info: MessageInfo,
    msg: ExecuteMsg,
) -> Result<Response, ContractError> {
    match msg {
        // Handle 'payable' functionalities
        ExecuteMsg::Receive(msg) => receive_cw20(deps, _env, info, msg),
        ExecuteMsg::CastVote {
            poll_id,
            vote,
            amount,
        } => cast_vote(deps, _env, info, poll_id, vote, amount),
        // Mark a poll as ended
        ExecuteMsg::EndPoll { poll_id } => end_poll(deps, _env, poll_id),
        // Execute the associated messages of a passed poll
        ExecuteMsg::ExecutePoll { poll_id } => execute_poll(deps, _env, poll_id),
        ExecuteMsg::ExpirePoll { poll_id } => expire_poll(deps, _env, poll_id),
        ExecuteMsg::RegisterContracts { whale_token } => register_contracts(deps, whale_token),
        ExecuteMsg::SnapshotPoll { poll_id } => snapshot_poll(deps, _env, poll_id),
        ExecuteMsg::WithdrawVotingTokens { amount } => withdraw_voting_tokens(deps, info, amount),
        ExecuteMsg::UpdateConfig {
            owner,
            quorum,
            threshold,
            voting_period,
            timelock_period,
            expiration_period,
            proposal_deposit,
            snapshot_period,
        } => update_config(
            deps,
            info,
            owner,
            quorum,
            threshold,
            voting_period,
            timelock_period,
            expiration_period,
            proposal_deposit,
            snapshot_period,
        ),
    }
}

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn query(deps: Deps, _env: Env, msg: QueryMsg) -> Result<Binary, ContractError> {
    match msg {
        QueryMsg::Config {} => Ok(to_binary(&query_config(deps)?)?),
        QueryMsg::State {} => Ok(to_binary(&query_state(deps)?)?),
        QueryMsg::Staker { address } => Ok(to_binary(&query_staker(deps, address)?)?),
        QueryMsg::Poll { poll_id } => Ok(to_binary(&query_poll(deps, poll_id)?)?),
        QueryMsg::Polls {
            filter,
            start_after,
            limit,
            order_by,
        } => Ok(to_binary(&query_polls(
            deps,
            filter,
            start_after,
            limit,
            order_by,
        )?)?),
        QueryMsg::Voters {
            poll_id,
            start_after,
            limit,
            order_by,
        } => Ok(to_binary(&query_voters(
            deps,
            poll_id,
            start_after,
            limit,
            order_by,
        )?)?),
    }
}

// ExecutionMsg handlers

pub fn register_contracts(deps: DepsMut, whale_token: String) -> Result<Response, ContractError> {
    let mut config: Config = config_read(deps.storage).load()?;
    if config.whale_token != CanonicalAddr::from(vec![]) {
        return Err(ContractError::Unauthorized {});
    }

    config.whale_token = deps.api.addr_canonicalize(&whale_token)?;
    config_store(deps.storage).save(&config)?;

    Ok(Response::default())
}

/// handler function invoked when the governance contract receives
/// a transaction. This is akin to a payable function in Solidity
pub fn receive_cw20(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
    cw20_msg: Cw20ReceiveMsg,
) -> Result<Response, ContractError> {
    // only asset contract can execute this message
    let config: Config = config_read(deps.storage).load()?;
    if config.whale_token != deps.api.addr_canonicalize(info.sender.as_str())? {
        return Err(ContractError::Unauthorized {});
    }

    match from_binary(&cw20_msg.msg) {
        Ok(Cw20HookMsg::StakeVotingTokens {}) => {
            let api = deps.api;
            stake_voting_tokens(deps, api.addr_validate(&cw20_msg.sender)?, cw20_msg.amount)
        }
        Ok(Cw20HookMsg::CreatePoll {
            title,
            description,
            link,
            execute_msgs,
        }) => create_poll(
            deps,
            env,
            cw20_msg.sender,
            cw20_msg.amount,
            title,
            description,
            link,
            execute_msgs,
        ),
        _ => Err(ContractError::DataShouldBeGiven {}),
    }
}

/// create a new poll
#[allow(clippy::too_many_arguments)]
pub fn create_poll(
    deps: DepsMut,
    env: Env,
    proposer: String,
    deposit_amount: Uint128,
    title: String,
    description: String,
    link: Option<String>,
    execute_msgs: Option<Vec<PollExecuteMsg>>,
) -> Result<Response, ContractError> {
    validate_poll_title(&title)?;
    validate_poll_description(&description)?;
    validate_poll_link(&link)?;

    let config: Config = config_store(deps.storage).load()?;
    if deposit_amount < config.proposal_deposit {
        return Err(ContractError::InsufficientProposalDeposit(
            config.proposal_deposit.u128(),
        ));
    }

    let mut state: State = state_store(deps.storage).load()?;
    let poll_id = state.poll_count + 1;

    // Increase poll count & total deposit amount
    state.poll_count += 1;
    state.total_deposit += deposit_amount;

    let mut data_list: Vec<ExecuteData> = vec![];
    let all_execute_data = if let Some(exe_msgs) = execute_msgs {
        for msgs in exe_msgs {
            let execute_data = ExecuteData {
                order: msgs.order,
                contract: deps.api.addr_canonicalize(&msgs.contract)?,
                msg: msgs.msg,
            };
            data_list.push(execute_data)
        }
        Some(data_list)
    } else {
        None
    };

    let sender_address_raw = deps.api.addr_canonicalize(&proposer)?;
    let new_poll = Poll {
        id: poll_id,
        creator: sender_address_raw,
        status: PollStatus::InProgress,
        yes_votes: Uint128::zero(),
        no_votes: Uint128::zero(),
        end_height: env.block.height + config.voting_period,
        title,
        description,
        link,
        execute_data: all_execute_data,
        deposit_amount,
        total_balance_at_end_poll: None,
        staked_amount: None,
    };

    poll_store(deps.storage).save(&poll_id.to_be_bytes(), &new_poll)?;
    poll_indexer_store(deps.storage, &PollStatus::InProgress)
        .save(&poll_id.to_be_bytes(), &true)?;

    state_store(deps.storage).save(&state)?;

    Ok(Response::new().add_attributes(vec![
        ("action", "create_poll"),
        (
            "creator",
            deps.api
                .addr_humanize(&new_poll.creator)?
                .to_string()
                .as_str(),
        ),
        ("poll_id", &poll_id.to_string()),
        ("end_height", new_poll.end_height.to_string().as_str()),
    ]))
}

/// end a poll
///
/// By default a Poll is considered rejected when ending. The weight of votes and the quorum of the vote is considered before declaring a Poll as passed.
/// Before the function completes, state is saved any leftover deposit amount is sent back to the poll creator and a response is returned.
pub fn end_poll(deps: DepsMut, env: Env, poll_id: u64) -> Result<Response, ContractError> {
    let mut a_poll: Poll = poll_store(deps.storage).load(&poll_id.to_be_bytes())?;

    if a_poll.status != PollStatus::InProgress {
        return Err(ContractError::PollNotInProgress {});
    }

    if a_poll.end_height > env.block.height {
        return Err(ContractError::PollVotingPeriod {});
    }

    let no = a_poll.no_votes.u128();
    let yes = a_poll.yes_votes.u128();

    let tallied_weight = yes + no;

    let mut poll_status = PollStatus::Rejected;
    #[allow(unused_mut)]
    let mut rejected_reason: &str;
    let mut passed = false;

    let mut messages: Vec<CosmosMsg> = vec![];
    let config: Config = config_read(deps.storage).load()?;
    let mut state: State = state_read(deps.storage).load()?;

    let (quorum, staked_weight) = if state.total_share.u128() == 0 {
        (Decimal::zero(), Uint128::zero())
    } else if let Some(staked_amount) = a_poll.staked_amount {
        (
            Decimal::from_ratio(tallied_weight, staked_amount),
            staked_amount,
        )
    } else {
        let staked_weight = query_token_balance(
            &deps.querier,
            deps.api.addr_humanize(&config.whale_token)?,
            deps.api.addr_humanize(&state.contract_addr)?,
        )?
        .checked_sub(state.total_deposit)?;

        (
            Decimal::from_ratio(tallied_weight, staked_weight),
            staked_weight,
        )
    };

    if tallied_weight == 0 || quorum < config.quorum {
        // Quorum: More than quorum of the total staked tokens at the end of the voting
        // period need to have participated in the vote.
        rejected_reason = "Quorum not reached";
    } else {
        if Decimal::from_ratio(yes, tallied_weight) > config.threshold {
            //Threshold: More than 50% of the tokens that participated in the vote
            // (after excluding “Abstain” votes) need to have voted in favor of the proposal (“Yes”).
            poll_status = PollStatus::Passed;
            rejected_reason = "Poll Passed";
            passed = true;
        } else {
            rejected_reason = "Threshold not reached";
        }

        // Refunds deposit only when quorum is reached
        if !a_poll.deposit_amount.is_zero() {
            messages.push(CosmosMsg::Wasm(WasmMsg::Execute {
                contract_addr: deps.api.addr_humanize(&config.whale_token)?.to_string(),
                funds: vec![],
                msg: to_binary(&Cw20ExecuteMsg::Transfer {
                    recipient: deps.api.addr_humanize(&a_poll.creator)?.to_string(),
                    amount: a_poll.deposit_amount,
                })?,
            }))
        }
    }

    // Decrease total deposit amount
    state.total_deposit = state.total_deposit.checked_sub(a_poll.deposit_amount)?;
    state_store(deps.storage).save(&state)?;

    // Update poll indexer
    poll_indexer_store(deps.storage, &PollStatus::InProgress).remove(&a_poll.id.to_be_bytes());
    poll_indexer_store(deps.storage, &poll_status).save(&a_poll.id.to_be_bytes(), &true)?;

    // Update poll status
    a_poll.status = poll_status;
    a_poll.total_balance_at_end_poll = Some(staked_weight);
    poll_store(deps.storage).save(&poll_id.to_be_bytes(), &a_poll)?;

    Ok(Response::new().add_messages(messages).add_attributes(vec![
        ("action", "end_poll"),
        ("poll_id", &poll_id.to_string()),
        ("rejected_reason", rejected_reason),
        ("passed", &passed.to_string()),
    ]))
}

/// execute_poll exposes the ability to execute the Messages which were defined on Polls creation if the Poll was deemed successful.
///
/// The fn first performs a number of checks to ensure the Poll indeed has passed and enough of an effective delay has elapsed
/// for the Messages to be executed. Provided these conditions are met the poll is declared in a Executed state
/// and the execution data that was provided when the poll was created is prepared as a number of CosmosMsg/WasmMsg(s) before being sent for execution.
///
///
/// It is important to note that execute poll only handles the execution of predefined messages
/// which are associated with a Passed poll. This ensures the actions taken by a successful Poll are
/// well known and predefined.
pub fn execute_poll(deps: DepsMut, env: Env, poll_id: u64) -> Result<Response, ContractError> {
    let config: Config = config_read(deps.storage).load()?;
    let mut a_poll: Poll = poll_store(deps.storage).load(&poll_id.to_be_bytes())?;

    if a_poll.status != PollStatus::Passed {
        return Err(ContractError::PollNotPassed {});
    }

    if a_poll.end_height + config.timelock_period > env.block.height {
        return Err(ContractError::TimelockNotExpired {});
    }

    poll_indexer_store(deps.storage, &PollStatus::Passed).remove(&poll_id.to_be_bytes());
    poll_indexer_store(deps.storage, &PollStatus::Executed).save(&poll_id.to_be_bytes(), &true)?;

    a_poll.status = PollStatus::Executed;
    poll_store(deps.storage).save(&poll_id.to_be_bytes(), &a_poll)?;

    let mut messages: Vec<CosmosMsg> = vec![];
    if let Some(all_msgs) = a_poll.execute_data {
        let mut msgs = all_msgs;
        msgs.sort();
        for msg in msgs {
            messages.push(CosmosMsg::Wasm(WasmMsg::Execute {
                contract_addr: deps.api.addr_humanize(&msg.contract)?.to_string(),
                msg: msg.msg,
                funds: vec![],
            }))
        }
    } else {
        return Err(ContractError::NoExecuteData {});
    }

    Ok(Response::new().add_messages(messages).add_attributes(vec![
        ("action", "execute_poll"),
        ("poll_id", poll_id.to_string().as_str()),
    ]))
}

// Voting
/// cast_vote exposes the end user side of a poll. Once a poll and its proposal is created,
/// any account which has some staked governance tokens can cast 1 vote for a given proposal.
///
/// Before a Vote is registered from a user a number of checks are performed; firstly that
/// the Poll exists and that it is currently in Progress. Accounts may only vote once
/// and the Account must have some staked governance tokens.
/// With all these conditions met, the account's casted vote is evaluated and both the vote and
/// a collection of info related to the Voter is stored in state. This registers both the actors
/// desired vote and also their information to prevent a second vote.
pub fn cast_vote(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
    poll_id: u64,
    vote: VoteOption,
    amount: Uint128,
) -> Result<Response, ContractError> {
    let sender_address_raw = deps.api.addr_canonicalize(info.sender.as_str())?;
    let config = config_read(deps.storage).load()?;
    let state = state_read(deps.storage).load()?;
    if poll_id == 0 || state.poll_count < poll_id {
        return Err(ContractError::PollNotFound {});
    }

    let mut a_poll: Poll = poll_store(deps.storage).load(&poll_id.to_be_bytes())?;
    if a_poll.status != PollStatus::InProgress || env.block.height > a_poll.end_height {
        return Err(ContractError::PollNotInProgress {});
    }

    // Check the voter already has a vote on the poll
    if poll_voter_read(deps.storage, poll_id)
        .load(sender_address_raw.as_slice())
        .is_ok()
    {
        return Err(ContractError::AlreadyVoted {});
    }

    let key = &sender_address_raw.as_slice();
    let mut token_manager = bank_read(deps.storage).may_load(key)?.unwrap_or_default();

    // convert share to amount
    let total_share = state.total_share;
    let total_balance = query_token_balance(
        &deps.querier,
        deps.api.addr_humanize(&config.whale_token)?,
        deps.api.addr_humanize(&state.contract_addr)?,
    )?
    .checked_sub(state.total_deposit)?;

    if token_manager
        .share
        .multiply_ratio(total_balance, total_share)
        < amount
    {
        return Err(ContractError::InsufficientStaked {});
    }

    // update tally info
    if VoteOption::Yes == vote {
        a_poll.yes_votes += amount;
    } else {
        a_poll.no_votes += amount;
    }

    let vote_info = VoterInfo {
        vote,
        balance: amount,
    };
    token_manager
        .locked_balance
        .push((poll_id, vote_info.clone()));
    bank_store(deps.storage).save(key, &token_manager)?;

    // store poll voter && and update poll data
    poll_voter_store(deps.storage, poll_id).save(sender_address_raw.as_slice(), &vote_info)?;

    // processing snapshot
    let time_to_end = a_poll.end_height - env.block.height;

    if time_to_end < config.snapshot_period && a_poll.staked_amount.is_none() {
        a_poll.staked_amount = Some(total_balance);
    }

    poll_store(deps.storage).save(&poll_id.to_be_bytes(), &a_poll)?;

    Ok(Response::new().add_attributes(vec![
        ("action", "cast_vote"),
        ("poll_id", poll_id.to_string().as_str()),
        ("amount", amount.to_string().as_str()),
        ("voter", info.sender.as_str()),
        ("vote_option", vote_info.vote.to_string().as_str()),
    ]))
}

/// ExpirePoll is used to make the poll as expired state for querying purpose
pub fn expire_poll(deps: DepsMut, env: Env, poll_id: u64) -> Result<Response, ContractError> {
    let config: Config = config_read(deps.storage).load()?;
    let mut a_poll: Poll = poll_store(deps.storage).load(&poll_id.to_be_bytes())?;

    if a_poll.status != PollStatus::Passed {
        return Err(ContractError::PollNotPassed {});
    }

    if a_poll.execute_data.is_none() {
        return Err(ContractError::NoExecuteData {});
    }

    if a_poll.end_height + config.expiration_period > env.block.height {
        return Err(ContractError::PollNotExpired {});
    }

    poll_indexer_store(deps.storage, &PollStatus::Passed).remove(&poll_id.to_be_bytes());
    poll_indexer_store(deps.storage, &PollStatus::Expired).save(&poll_id.to_be_bytes(), &true)?;

    a_poll.status = PollStatus::Expired;
    poll_store(deps.storage).save(&poll_id.to_be_bytes(), &a_poll)?;

    Ok(Response::new().add_attributes(vec![
        ("action", "expire_poll"),
        ("poll_id", poll_id.to_string().as_str()),
    ]))
}

// Query Handlers

/// query_config allows for the query of the currently set configuration values
/// which influence Polls such as the quorum needed and the minimum voting peroid before a poll can be ended
fn query_config(deps: Deps) -> Result<ConfigResponse, ContractError> {
    let config: Config = config_read(deps.storage).load()?;
    Ok(ConfigResponse {
        owner: deps.api.addr_humanize(&config.owner)?.to_string(),
        whale_token: deps.api.addr_humanize(&config.whale_token)?.to_string(),
        quorum: config.quorum,
        threshold: config.threshold,
        voting_period: config.voting_period,
        timelock_period: config.timelock_period,
        expiration_period: config.expiration_period,
        proposal_deposit: config.proposal_deposit,
        snapshot_period: config.snapshot_period,
    })
}

/// query_state allows for the query of dynamic state values such as the poll count and how much has been deposited
fn query_state(deps: Deps) -> Result<StateResponse, ContractError> {
    let state: State = state_read(deps.storage).load()?;
    Ok(StateResponse {
        poll_count: state.poll_count,
        total_share: state.total_share,
        total_deposit: state.total_deposit,
    })
}

/// query_poll allows for the query of a given poll by supplying its poll_id
fn query_poll(deps: Deps, poll_id: u64) -> Result<PollResponse, ContractError> {
    let poll = match poll_read(deps.storage).may_load(&poll_id.to_be_bytes())? {
        Some(poll) => Some(poll),
        None => return Err(ContractError::PollNotFound {}),
    }
    .unwrap();

    let mut data_list: Vec<PollExecuteMsg> = vec![];

    Ok(PollResponse {
        id: poll.id,
        creator: deps.api.addr_humanize(&poll.creator)?.to_string(),
        status: poll.status,
        end_height: poll.end_height,
        title: poll.title,
        description: poll.description,
        link: poll.link,
        deposit_amount: poll.deposit_amount,
        execute_data: if let Some(exe_msgs) = poll.execute_data.clone() {
            for msg in exe_msgs {
                let execute_data = PollExecuteMsg {
                    order: msg.order,
                    contract: deps.api.addr_humanize(&msg.contract)?.to_string(),
                    msg: msg.msg,
                };
                data_list.push(execute_data)
            }
            Some(data_list)
        } else {
            None
        },
        yes_votes: poll.yes_votes,
        no_votes: poll.no_votes,
        staked_amount: poll.staked_amount,
        total_balance_at_end_poll: poll.total_balance_at_end_poll,
    })
}

fn query_polls(
    deps: Deps,
    filter: Option<PollStatus>,
    start_after: Option<u64>,
    limit: Option<u32>,
    order_by: Option<OrderBy>,
) -> Result<PollsResponse, ContractError> {
    let polls = read_polls(deps.storage, filter, start_after, limit, order_by)?;

    let poll_responses: StdResult<Vec<PollResponse>> = polls
        .iter()
        .map(|poll| {
            Ok(PollResponse {
                id: poll.id,
                creator: deps.api.addr_humanize(&poll.creator)?.to_string(),
                status: poll.status.clone(),
                end_height: poll.end_height,
                title: poll.title.to_string(),
                description: poll.description.to_string(),
                link: poll.link.clone(),
                deposit_amount: poll.deposit_amount,
                execute_data: if let Some(exe_msgs) = poll.execute_data.clone() {
                    let mut data_list: Vec<PollExecuteMsg> = vec![];

                    for msg in exe_msgs {
                        let execute_data = PollExecuteMsg {
                            order: msg.order,
                            contract: deps.api.addr_humanize(&msg.contract)?.to_string(),
                            msg: msg.msg,
                        };
                        data_list.push(execute_data)
                    }
                    Some(data_list)
                } else {
                    None
                },
                yes_votes: poll.yes_votes,
                no_votes: poll.no_votes,
                staked_amount: poll.staked_amount,
                total_balance_at_end_poll: poll.total_balance_at_end_poll,
            })
        })
        .collect();

    Ok(PollsResponse {
        polls: poll_responses?,
    })
}

fn query_voters(
    deps: Deps,
    poll_id: u64,
    start_after: Option<String>,
    limit: Option<u32>,
    order_by: Option<OrderBy>,
) -> Result<VotersResponse, ContractError> {
    let poll: Poll = match poll_read(deps.storage).may_load(&poll_id.to_be_bytes())? {
        Some(poll) => Some(poll),
        None => return Err(ContractError::PollNotFound {}),
    }
    .unwrap();

    let voters = if poll.status != PollStatus::InProgress {
        vec![]
    } else if let Some(start_after) = start_after {
        read_poll_voters(
            deps.storage,
            poll_id,
            Some(deps.api.addr_canonicalize(&start_after)?),
            limit,
            order_by,
        )?
    } else {
        read_poll_voters(deps.storage, poll_id, None, limit, order_by)?
    };

    let voters_response: StdResult<Vec<VotersResponseItem>> = voters
        .iter()
        .map(|voter_info| {
            Ok(VotersResponseItem {
                voter: deps.api.addr_humanize(&voter_info.0)?.to_string(),
                vote: voter_info.1.vote.clone(),
                balance: voter_info.1.balance,
            })
        })
        .collect();

    Ok(VotersResponse {
        voters: voters_response?,
    })
}

/// SnapshotPoll is used to take a snapshot of the staked amount for quorum calculation
pub fn snapshot_poll(deps: DepsMut, env: Env, poll_id: u64) -> Result<Response, ContractError> {
    let config: Config = config_read(deps.storage).load()?;
    let mut a_poll: Poll = poll_store(deps.storage).load(&poll_id.to_be_bytes())?;

    if a_poll.status != PollStatus::InProgress {
        return Err(ContractError::PollNotInProgress {});
    }

    let time_to_end = a_poll.end_height - env.block.height;

    if time_to_end > config.snapshot_period {
        return Err(ContractError::SnapshotHeight {});
    }

    if a_poll.staked_amount.is_some() {
        return Err(ContractError::SnapshotAlreadyOccurred {});
    }

    // store the current staked amount for quorum calculation
    let state: State = state_store(deps.storage).load()?;

    let staked_amount = query_token_balance(
        &deps.querier,
        deps.api.addr_humanize(&config.whale_token)?,
        deps.api.addr_humanize(&state.contract_addr)?,
    )?
    .checked_sub(state.total_deposit)?;

    a_poll.staked_amount = Some(staked_amount);

    poll_store(deps.storage).save(&poll_id.to_be_bytes(), &a_poll)?;

    Ok(Response::new().add_attributes(vec![
        attr("action", "snapshot_poll"),
        attr("poll_id", poll_id.to_string().as_str()),
        attr("staked_amount", staked_amount.to_string().as_str()),
    ]))
}

#[allow(clippy::too_many_arguments)]
pub fn update_config(
    deps: DepsMut,
    info: MessageInfo,
    owner: Option<String>,
    quorum: Option<Decimal>,
    threshold: Option<Decimal>,
    voting_period: Option<u64>,
    timelock_period: Option<u64>,
    expiration_period: Option<u64>,
    proposal_deposit: Option<Uint128>,
    snapshot_period: Option<u64>,
) -> Result<Response, ContractError> {
    let api = deps.api;
    config_store(deps.storage).update(|mut config| {
        if config.owner != api.addr_canonicalize(info.sender.as_str())? {
            return Err(ContractError::Unauthorized {});
        }

        if let Some(owner) = owner {
            config.owner = api.addr_canonicalize(&owner)?;
        }

        if let Some(quorum) = quorum {
            config.quorum = quorum;
        }

        if let Some(threshold) = threshold {
            config.threshold = threshold;
        }

        if let Some(voting_period) = voting_period {
            config.voting_period = voting_period;
        }

        if let Some(timelock_period) = timelock_period {
            config.timelock_period = timelock_period;
        }

        if let Some(expiration_period) = expiration_period {
            config.expiration_period = expiration_period;
        }

        if let Some(proposal_deposit) = proposal_deposit {
            config.proposal_deposit = proposal_deposit;
        }

        if let Some(period) = snapshot_period {
            config.snapshot_period = period;
        }

        Ok(config)
    })?;

    Ok(Response::new().add_attributes(vec![("action", "update_config")]))
}
