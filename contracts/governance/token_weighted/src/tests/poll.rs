use cosmwasm_std::testing::{mock_env, mock_info, MOCK_CONTRACT_ADDR};
use cosmwasm_std::{
    attr, coins, from_binary, to_binary, Addr, Api, CosmosMsg, Deps, DepsMut, Response, SubMsg,
    Uint128, WasmMsg,
};
use cw20::{Cw20ExecuteMsg, Cw20ReceiveMsg};
use terraswap::querier::query_token_balance;

use crate::contract::{execute, query};
use crate::{ExecuteMsg, QueryMsg};
use crate::state::{
    bank_read, poll_voter_read, state_read, Cw20HookMsg, OrderBy, PollExecuteMsg, PollResponse,
    PollStatus, PollsResponse, StakerResponse, State, VoteOption, VoterInfo, VotersResponse,
};
use crate::tests::common::{
    mock_env_height, DEFAULT_EXPIRATION_PERIOD, DEFAULT_PROPOSAL_DEPOSIT, DEFAULT_TIMELOCK_PERIOD,
    DEFAULT_VOTING_PERIOD, TEST_CREATOR, TEST_VOTER, TEST_VOTER_2, TEST_VOTER_3, VOTING_TOKEN,
};
use crate::tests::mock_querier::mock_dependencies;
use crate::tests::{common, instantiate};
use crate::ContractError;

pub fn mock_register_voting_token(deps: DepsMut) {
    let info = mock_info(TEST_CREATOR, &[]);
    let msg = ExecuteMsg::RegisterContracts {
        whale_token: VOTING_TOKEN.to_string(),
    };
    let _res = execute(deps, mock_env(), info, msg)
        .expect("contract successfully handles RegisterContracts");
}

pub fn create_poll_msg(
    title: String,
    description: String,
    link: Option<String>,
    execute_msg: Option<Vec<PollExecuteMsg>>,
) -> ExecuteMsg {
    ExecuteMsg::Receive(Cw20ReceiveMsg {
        sender: TEST_CREATOR.to_string(),
        amount: Uint128::from(DEFAULT_PROPOSAL_DEPOSIT),
        msg: to_binary(&Cw20HookMsg::CreatePoll {
            title,
            description,
            link,
            execute_msgs: execute_msg,
        })
        .unwrap(),
    })
}

/**
 * Assert helper to confirm the expected create_poll response
 */
pub fn assert_create_poll_result(
    poll_id: u64,
    end_height: u64,
    creator: &str,
    execute_res: Response,
    deps: Deps,
) {
    assert_eq!(
        execute_res.attributes,
        vec![
            attr("action", "create_poll"),
            attr("creator", creator),
            attr("poll_id", poll_id.to_string()),
            attr("end_height", end_height.to_string()),
        ]
    );

    //confirm poll count
    let state: State = state_read(deps.storage).load().unwrap();
    assert_eq!(
        state,
        State {
            contract_addr: deps.api.addr_canonicalize(MOCK_CONTRACT_ADDR).unwrap(),
            poll_count: 1,
            total_share: Uint128::zero(),
            total_deposit: Uint128::from(DEFAULT_PROPOSAL_DEPOSIT),
        }
    );
}

/**
 * Assert helper to confirm the expected staked tokens response
 */
pub fn assert_stake_tokens_result(
    total_share: u128,
    total_deposit: u128,
    new_share: u128,
    poll_count: u64,
    execute_res: Response,
    deps: Deps,
) {
    assert_eq!(
        execute_res.attributes.get(2).expect("no log"),
        &attr("share", new_share.to_string())
    );

    let state: State = state_read(deps.storage).load().unwrap();
    assert_eq!(
        state,
        State {
            contract_addr: deps.api.addr_canonicalize(MOCK_CONTRACT_ADDR).unwrap(),
            poll_count,
            total_share: Uint128::from(total_share),
            total_deposit: Uint128::from(total_deposit),
        }
    );
}

/**
 * Assert helper to confirm the expected cast_vote response
 */
pub fn assert_cast_vote_success(
    voter: &str,
    amount: u128,
    poll_id: u64,
    vote_option: VoteOption,
    execute_res: Response,
) {
    assert_eq!(
        execute_res.attributes,
        vec![
            attr("action", "cast_vote"),
            attr("poll_id", poll_id.to_string()),
            attr("amount", amount.to_string()),
            attr("voter", voter),
            attr("vote_option", vote_option.to_string()),
        ]
    );
}

/**
 * Tests related to the Execution of messages defined with the proposal/poll
 */
#[test]
fn add_several_execute_msgs() {
    let mut deps = mock_dependencies(&[]);
    instantiate::mock_instantiate(deps.as_mut());
    mock_register_voting_token(deps.as_mut());
    let info = mock_info(VOTING_TOKEN, &[]);
    let env = common::mock_env_height(0, 10000);

    let exec_msg_bz = to_binary(&Cw20ExecuteMsg::Burn {
        amount: Uint128::new(123),
    })
    .unwrap();

    let exec_msg_bz2 = to_binary(&Cw20ExecuteMsg::Burn {
        amount: Uint128::new(12),
    })
    .unwrap();

    let exec_msg_bz3 = to_binary(&Cw20ExecuteMsg::Burn {
        amount: Uint128::new(1),
    })
    .unwrap();

    // push two execute msgs to the list
    let execute_msgs: Vec<PollExecuteMsg> = vec![
        PollExecuteMsg {
            order: 1u64,
            contract: VOTING_TOKEN.to_string(),
            msg: exec_msg_bz,
        },
        PollExecuteMsg {
            order: 3u64,
            contract: VOTING_TOKEN.to_string(),
            msg: exec_msg_bz3,
        },
        PollExecuteMsg {
            order: 2u64,
            contract: VOTING_TOKEN.to_string(),
            msg: exec_msg_bz2,
        },
    ];

    let msg = create_poll_msg(
        "test".to_string(),
        "test".to_string(),
        None,
        Some(execute_msgs.clone()),
    );

    let execute_res = execute(deps.as_mut(), env.clone(), info, msg).unwrap();
    assert_create_poll_result(
        1,
        env.block.height + DEFAULT_VOTING_PERIOD,
        TEST_CREATOR,
        execute_res,
        deps.as_ref(),
    );

    let res = query(deps.as_ref(), mock_env(), QueryMsg::Poll { poll_id: 1 }).unwrap();
    let value: PollResponse = from_binary(&res).unwrap();

    let response_execute_data = value.execute_data.unwrap();
    assert_eq!(response_execute_data.len(), 3);
    assert_eq!(response_execute_data, execute_msgs);
}

#[test]
fn successful_create_poll() {
    let mut deps = mock_dependencies(&[]);
    instantiate::mock_instantiate(deps.as_mut());
    mock_register_voting_token(deps.as_mut());
    let env = mock_env_height(0, 10000);
    let info = mock_info(VOTING_TOKEN, &[]);

    let msg = create_poll_msg("test".to_string(), "test".to_string(), None, None);

    let execute_res = execute(deps.as_mut(), env.clone(), info, msg).unwrap();
    assert_create_poll_result(
        1,
        env.block.height + DEFAULT_VOTING_PERIOD,
        TEST_CREATOR,
        execute_res,
        deps.as_ref(),
    );
}

#[test]
fn create_poll_no_quorum() {
    let mut deps = mock_dependencies(&[]);
    instantiate::mock_instantiate(deps.as_mut());
    mock_register_voting_token(deps.as_mut());

    let info = mock_info(VOTING_TOKEN, &[]);
    let env = mock_env_height(0, 10000);

    let msg = create_poll_msg("test".to_string(), "test".to_string(), None, None);

    let execute_res = execute(deps.as_mut(), env, info, msg).unwrap();
    assert_create_poll_result(
        1,
        DEFAULT_VOTING_PERIOD,
        TEST_CREATOR,
        execute_res,
        deps.as_ref(),
    );
}

#[test]
fn fails_create_poll_invalid_short_title() {
    let mut deps = mock_dependencies(&[]);
    instantiate::mock_instantiate(deps.as_mut());
    mock_register_voting_token(deps.as_mut());

    let msg = create_poll_msg("a".to_string(), "valid description".to_string(), None, None);
    let info = mock_info(VOTING_TOKEN, &[]);
    match execute(deps.as_mut(), mock_env(), info.clone(), msg) {
        Ok(_) => panic!("Must return error"),
        Err(ContractError::PollTitleInvalidShort(..)) => (),
        Err(_) => panic!("Unknown error"),
    }
}

#[test]
fn fails_create_poll_invalid_long_title() {
    let mut deps = mock_dependencies(&[]);
    instantiate::mock_instantiate(deps.as_mut());
    mock_register_voting_token(deps.as_mut());

    let invalid_title =
        String::from("Lorem ipsum dolor sit amet, consectetur adipiscing elit, sed do e");
    let msg = create_poll_msg(invalid_title, "valid description".to_string(), None, None);
    let info = mock_info(VOTING_TOKEN, &[]);
    match execute(deps.as_mut(), mock_env(), info.clone(), msg) {
        Ok(_) => panic!("Must return error"),
        Err(ContractError::PollTitleInvalidLong(..)) => (),
        Err(_) => panic!("Unknown error"),
    }
}

#[test]
fn fails_create_poll_invalid_short_description() {
    let mut deps = mock_dependencies(&[]);
    instantiate::mock_instantiate(deps.as_mut());
    mock_register_voting_token(deps.as_mut());

    let msg = create_poll_msg("valid title".to_string(), "abc".to_string(), None, None);
    let info = mock_info(VOTING_TOKEN, &[]);
    match execute(deps.as_mut(), mock_env(), info.clone(), msg) {
        Ok(_) => panic!("Must return error"),
        Err(ContractError::PollDescriptionInvalidShort(..)) => (),
        Err(_) => panic!("Unknown error"),
    }
}

#[test]
fn fails_create_poll_invalid_long_description() {
    let mut deps = mock_dependencies(&[]);
    instantiate::mock_instantiate(deps.as_mut());
    mock_register_voting_token(deps.as_mut());

    let s1 = "Lorem ipsum dolor sit amet, consectetur adipiscing elit, sed do eiusmod \
    tempor incididunt ut labore et dolore magna aliqua. Ut enim ad minim veniam, quis nostrud \
    exercitation ullamco laboris nisi ut aliquip ex ea commodo consequat. Duis aute irure dolor \
    in reprehenderit in voluptate velit esse cillum dolore eu fugiat nulla pariatur. Excepteur \
    sint occaecat cupidatat non proident, sunt in culpa qui officia deserunt mollit anim id est \
    laborum.";
    let s2 = "Section 1.10.32 of de Finibus Bonorum et Malorum, written by Cicero in 45 BC";
    let s3 = "Sed ut perspiciatis unde omnis iste natus error sit voluptatem accusantium \
    doloremque laudantium, totam rem aperiam, eaque ipsa quae ab illo inventore veritatis et quasi \
    architecto beatae vitae dicta sunt explicabo. Nemo enim ipsam voluptatem quia voluptas sit \
    aspernatur aut odit aut fugit, sed quia consequuntur magni dolores eos qui ratione voluptatem \
    sequi nesciunt. Neque porro quisquam est, qui dolorem ipsum quia dolor sit amet, consectetur, \
    adipisci velit, sed quia non numquam eius modi tempora ";
    let long_desc = s1.to_owned().clone() + s2 + s3;

    let msg = create_poll_msg("valid title".to_string(), long_desc, None, None);
    let info = mock_info(VOTING_TOKEN, &[]);

    match execute(deps.as_mut(), mock_env(), info, msg) {
        Ok(_) => panic!("Must return error"),
        Err(ContractError::PollDescriptionInvalidLong(..)) => (),
        Err(_) => panic!("Unknown error"),
    }
}

#[test]
fn fails_create_poll_invalid_short_link() {
    let mut deps = mock_dependencies(&[]);
    instantiate::mock_instantiate(deps.as_mut());
    mock_register_voting_token(deps.as_mut());

    let msg = create_poll_msg(
        "valid title".to_string(),
        "valid description".to_string(),
        Some("http://hih".to_string()),
        None,
    );
    let info = mock_info(VOTING_TOKEN, &[]);
    match execute(deps.as_mut(), mock_env(), info.clone(), msg) {
        Ok(_) => panic!("Must return error"),
        Err(ContractError::PollLinkInvalidShort(..)) => (),
        Err(_) => panic!("Unknown error"),
    }
}

#[test]
fn fails_create_poll_invalid_long_link() {
    let mut deps = mock_dependencies(&[]);
    instantiate::mock_instantiate(deps.as_mut());
    mock_register_voting_token(deps.as_mut());

    let long_link = String::from("https://whaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaale.com");
    let msg = create_poll_msg(
        "valid title".to_string(),
        "valid description".to_string(),
        Some(long_link),
        None,
    );
    let info = mock_info(VOTING_TOKEN, &[]);

    match execute(deps.as_mut(), mock_env(), info, msg) {
        Ok(_) => panic!("Must return error"),
        Err(ContractError::PollLinkInvalidLong(..)) => (),
        Err(_) => panic!("Unknown error"),
    }
}

#[test]
fn fails_create_poll_invalid_deposit() {
    let mut deps = mock_dependencies(&[]);
    instantiate::mock_instantiate(deps.as_mut());
    mock_register_voting_token(deps.as_mut());

    let msg = ExecuteMsg::Receive(Cw20ReceiveMsg {
        sender: TEST_CREATOR.to_string(),
        amount: Uint128::from(DEFAULT_PROPOSAL_DEPOSIT - 1),
        msg: to_binary(&Cw20HookMsg::CreatePoll {
            title: "TESTTEST".to_string(),
            description: "TESTTEST".to_string(),
            link: None,
            execute_msgs: None,
        })
        .unwrap(),
    });

    let info = mock_info(VOTING_TOKEN, &[]);
    match execute(deps.as_mut(), mock_env(), info, msg) {
        Ok(_) => panic!("Must return error"),
        Err(ContractError::InsufficientProposalDeposit(DEFAULT_PROPOSAL_DEPOSIT)) => (),
        Err(_) => panic!("Unknown error"),
    }
}

#[test]
fn expire_poll() {
    const POLL_START_HEIGHT: u64 = 1000;
    const POLL_ID: u64 = 1;
    let stake_amount = 1000;

    let mut deps = mock_dependencies(&coins(1000, VOTING_TOKEN));
    instantiate::mock_instantiate(deps.as_mut());
    mock_register_voting_token(deps.as_mut());
    let mut creator_env = common::mock_env_height(POLL_START_HEIGHT, 10000);
    let creator_info = mock_info(VOTING_TOKEN, &coins(2, VOTING_TOKEN));

    let exec_msg_bz = to_binary(&Cw20ExecuteMsg::Burn {
        amount: Uint128::new(123),
    })
    .unwrap();
    let execute_msgs: Vec<PollExecuteMsg> = vec![PollExecuteMsg {
        order: 1u64,
        contract: VOTING_TOKEN.to_string(),
        msg: exec_msg_bz,
    }];
    let msg = create_poll_msg(
        "test".to_string(),
        "test".to_string(),
        None,
        Some(execute_msgs),
    );

    let execute_res = execute(
        deps.as_mut(),
        creator_env.clone(),
        creator_info.clone(),
        msg,
    )
    .unwrap();

    assert_create_poll_result(
        1,
        creator_env.block.height + DEFAULT_VOTING_PERIOD,
        TEST_CREATOR,
        execute_res,
        deps.as_ref(),
    );

    deps.querier.with_token_balances(&[(
        &VOTING_TOKEN.to_string(),
        &[(
            &MOCK_CONTRACT_ADDR.to_string(),
            &Uint128::from((stake_amount + DEFAULT_PROPOSAL_DEPOSIT) as u128),
        )],
    )]);

    let msg = ExecuteMsg::Receive(Cw20ReceiveMsg {
        sender: TEST_VOTER.to_string(),
        amount: Uint128::from(stake_amount as u128),
        msg: to_binary(&Cw20HookMsg::StakeVotingTokens {}).unwrap(),
    });

    let info = mock_info(VOTING_TOKEN, &[]);
    let execute_res = execute(deps.as_mut(), mock_env(), info, msg).unwrap();
    assert_stake_tokens_result(
        stake_amount,
        DEFAULT_PROPOSAL_DEPOSIT,
        stake_amount,
        1,
        execute_res,
        deps.as_ref(),
    );

    let msg = ExecuteMsg::CastVote {
        poll_id: 1,
        vote: VoteOption::Yes,
        amount: Uint128::from(stake_amount),
    };
    let env = common::mock_env_height(POLL_START_HEIGHT, 10000);
    let info = mock_info(TEST_VOTER, &[]);
    let execute_res = execute(deps.as_mut(), env, info, msg).unwrap();

    assert_eq!(
        execute_res.attributes,
        vec![
            attr("action", "cast_vote"),
            attr("poll_id", POLL_ID.to_string().as_str()),
            attr("amount", "1000"),
            attr("voter", TEST_VOTER),
            attr("vote_option", "yes"),
        ]
    );

    // Poll is not in passed status
    creator_env.block.height += DEFAULT_TIMELOCK_PERIOD;
    let msg = ExecuteMsg::ExpirePoll { poll_id: 1 };
    let execute_res = execute(
        deps.as_mut(),
        creator_env.clone(),
        creator_info.clone(),
        msg,
    );
    match execute_res {
        Err(ContractError::PollNotPassed {}) => (),
        _ => panic!("DO NOT ENTER HERE"),
    }

    let msg = ExecuteMsg::EndPoll { poll_id: 1 };
    let execute_res = execute(
        deps.as_mut(),
        creator_env.clone(),
        creator_info.clone(),
        msg,
    )
    .unwrap();

    assert_eq!(
        execute_res.attributes,
        vec![
            attr("action", "end_poll"),
            attr("poll_id", "1"),
            attr("rejected_reason", "Poll Passed"),
            attr("passed", "true"),
        ]
    );
    assert_eq!(
        execute_res.messages,
        vec![SubMsg::new(CosmosMsg::Wasm(WasmMsg::Execute {
            contract_addr: VOTING_TOKEN.to_string(),
            msg: to_binary(&Cw20ExecuteMsg::Transfer {
                recipient: TEST_CREATOR.to_string(),
                amount: Uint128::from(DEFAULT_PROPOSAL_DEPOSIT),
            })
            .unwrap(),
            funds: vec![],
        }))]
    );

    // Expiration period has not been passed
    let msg = ExecuteMsg::ExpirePoll { poll_id: 1 };
    let execute_res = execute(
        deps.as_mut(),
        creator_env.clone(),
        creator_info.clone(),
        msg,
    );
    match execute_res {
        Err(ContractError::PollNotExpired {}) => (),
        _ => panic!("DO NOT ENTER HERE"),
    }

    creator_env.block.height += DEFAULT_EXPIRATION_PERIOD;
    let msg = ExecuteMsg::ExpirePoll { poll_id: 1 };
    let _execute_res = execute(deps.as_mut(), creator_env, creator_info, msg).unwrap();

    let res = query(deps.as_ref(), mock_env(), QueryMsg::Poll { poll_id: 1 }).unwrap();
    let poll_res: PollResponse = from_binary(&res).unwrap();
    assert_eq!(poll_res.status, PollStatus::Expired);

    let res = query(
        deps.as_ref(),
        mock_env(),
        QueryMsg::Polls {
            filter: Some(PollStatus::Expired),
            start_after: None,
            limit: None,
            order_by: Some(OrderBy::Desc),
        },
    )
    .unwrap();
    let polls_res: PollsResponse = from_binary(&res).unwrap();
    assert_eq!(polls_res.polls[0], poll_res);
}

/**
 * end_poll Tests
 */
#[test]
fn fails_end_poll_before_end_height() {
    let mut deps = mock_dependencies(&[]);
    instantiate::mock_instantiate(deps.as_mut());
    mock_register_voting_token(deps.as_mut());
    let env = common::mock_env_height(0, 10000);
    let info = mock_info(VOTING_TOKEN, &[]);

    let msg = create_poll_msg("test".to_string(), "test".to_string(), None, None);

    let execute_res = execute(deps.as_mut(), env, info, msg).unwrap();
    assert_create_poll_result(
        1,
        DEFAULT_VOTING_PERIOD,
        TEST_CREATOR,
        execute_res,
        deps.as_ref(),
    );

    let res = query(deps.as_ref(), mock_env(), QueryMsg::Poll { poll_id: 1 }).unwrap();
    let value: PollResponse = from_binary(&res).unwrap();
    assert_eq!(DEFAULT_VOTING_PERIOD, value.end_height);

    let msg = ExecuteMsg::EndPoll { poll_id: 1 };
    let env = common::mock_env_height(0, 10000);
    let info = mock_info(TEST_CREATOR, &[]);
    let execute_res = execute(deps.as_mut(), env, info, msg);

    match execute_res {
        Ok(_) => panic!("Must return error"),
        Err(ContractError::PollVotingPeriod {}) => (),
        Err(e) => panic!("Unexpected error: {:?}", e),
    }
}

#[test]
fn successful_end_poll() {
    const POLL_START_HEIGHT: u64 = 1000;
    const POLL_ID: u64 = 1;
    let stake_amount = 1000;

    let mut deps = mock_dependencies(&coins(1000, VOTING_TOKEN));
    instantiate::mock_instantiate(deps.as_mut());
    mock_register_voting_token(deps.as_mut());
    let mut creator_env = common::mock_env_height(POLL_START_HEIGHT, 10000);
    let mut creator_info = mock_info(VOTING_TOKEN, &coins(2, VOTING_TOKEN));

    let exec_msg_bz = to_binary(&Cw20ExecuteMsg::Burn {
        amount: Uint128::new(123),
    })
    .unwrap();

    let exec_msg_bz2 = to_binary(&Cw20ExecuteMsg::Burn {
        amount: Uint128::new(12),
    })
    .unwrap();

    let exec_msg_bz3 = to_binary(&Cw20ExecuteMsg::Burn {
        amount: Uint128::new(1),
    })
    .unwrap();

    //add three messages with different order
    let execute_msgs: Vec<PollExecuteMsg> = vec![
        PollExecuteMsg {
            order: 3u64,
            contract: VOTING_TOKEN.to_string(),
            msg: exec_msg_bz3.clone(),
        },
        PollExecuteMsg {
            order: 2u64,
            contract: VOTING_TOKEN.to_string(),
            msg: exec_msg_bz2.clone(),
        },
        PollExecuteMsg {
            order: 1u64,
            contract: VOTING_TOKEN.to_string(),
            msg: exec_msg_bz.clone(),
        },
    ];

    let msg = create_poll_msg(
        "test".to_string(),
        "test".to_string(),
        None,
        Some(execute_msgs),
    );

    let execute_res = execute(
        deps.as_mut(),
        creator_env.clone(),
        creator_info.clone(),
        msg,
    )
    .unwrap();

    assert_create_poll_result(
        1,
        creator_env.block.height + DEFAULT_VOTING_PERIOD,
        TEST_CREATOR,
        execute_res,
        deps.as_ref(),
    );

    deps.querier.with_token_balances(&[(
        &VOTING_TOKEN.to_string(),
        &[(
            &MOCK_CONTRACT_ADDR.to_string(),
            &Uint128::from((stake_amount + DEFAULT_PROPOSAL_DEPOSIT) as u128),
        )],
    )]);

    let msg = ExecuteMsg::Receive(Cw20ReceiveMsg {
        sender: TEST_VOTER.to_string(),
        amount: Uint128::from(stake_amount as u128),
        msg: to_binary(&Cw20HookMsg::StakeVotingTokens {}).unwrap(),
    });

    let info = mock_info(VOTING_TOKEN, &[]);
    let execute_res = execute(deps.as_mut(), mock_env(), info, msg).unwrap();
    assert_stake_tokens_result(
        stake_amount,
        DEFAULT_PROPOSAL_DEPOSIT,
        stake_amount,
        1,
        execute_res,
        deps.as_ref(),
    );

    let msg = ExecuteMsg::CastVote {
        poll_id: 1,
        vote: VoteOption::Yes,
        amount: Uint128::from(stake_amount),
    };
    let env = common::mock_env_height(POLL_START_HEIGHT, 10000);
    let info = mock_info(TEST_VOTER, &[]);
    let execute_res = execute(deps.as_mut(), env, info, msg).unwrap();

    assert_eq!(
        execute_res.attributes,
        vec![
            attr("action", "cast_vote"),
            attr("poll_id", POLL_ID.to_string().as_str()),
            attr("amount", "1000"),
            attr("voter", TEST_VOTER),
            attr("vote_option", "yes"),
        ]
    );

    // not in passed status
    let msg = ExecuteMsg::ExecutePoll { poll_id: 1 };
    let execute_res = execute(
        deps.as_mut(),
        creator_env.clone(),
        creator_info.clone(),
        msg,
    )
    .unwrap_err();
    match execute_res {
        ContractError::PollNotPassed {} => (),
        _ => panic!("DO NOT ENTER HERE"),
    }

    creator_info.sender = Addr::unchecked(TEST_CREATOR);
    creator_env.block.height += DEFAULT_VOTING_PERIOD;

    let msg = ExecuteMsg::EndPoll { poll_id: 1 };
    let execute_res = execute(
        deps.as_mut(),
        creator_env.clone(),
        creator_info.clone(),
        msg,
    )
    .unwrap();

    assert_eq!(
        execute_res.attributes,
        vec![
            attr("action", "end_poll"),
            attr("poll_id", "1"),
            attr("rejected_reason", "Poll Passed"),
            attr("passed", "true"),
        ]
    );
    assert_eq!(
        execute_res.messages,
        vec![SubMsg::new(CosmosMsg::Wasm(WasmMsg::Execute {
            contract_addr: VOTING_TOKEN.to_string(),
            msg: to_binary(&Cw20ExecuteMsg::Transfer {
                recipient: TEST_CREATOR.to_string(),
                amount: Uint128::from(DEFAULT_PROPOSAL_DEPOSIT),
            })
            .unwrap(),
            funds: vec![],
        }))]
    );

    // End poll will withdraw deposit balance
    deps.querier.with_token_balances(&[(
        &VOTING_TOKEN.to_string(),
        &[(
            &MOCK_CONTRACT_ADDR.to_string(),
            &Uint128::from(stake_amount as u128),
        )],
    )]);

    // timelock_period has not expired
    let msg = ExecuteMsg::ExecutePoll { poll_id: 1 };
    let execute_res = execute(
        deps.as_mut(),
        creator_env.clone(),
        creator_info.clone(),
        msg,
    )
    .unwrap_err();
    match execute_res {
        ContractError::TimelockNotExpired {} => (),
        _ => panic!("DO NOT ENTER HERE"),
    }

    creator_env.block.height += DEFAULT_TIMELOCK_PERIOD;
    let msg = ExecuteMsg::ExecutePoll { poll_id: 1 };
    let execute_res = execute(deps.as_mut(), creator_env, creator_info, msg).unwrap();
    assert_eq!(
        execute_res.messages,
        vec![
            SubMsg::new(CosmosMsg::Wasm(WasmMsg::Execute {
                contract_addr: VOTING_TOKEN.to_string(),
                msg: exec_msg_bz,
                funds: vec![],
            })),
            SubMsg::new(CosmosMsg::Wasm(WasmMsg::Execute {
                contract_addr: VOTING_TOKEN.to_string(),
                msg: exec_msg_bz2,
                funds: vec![],
            })),
            SubMsg::new(CosmosMsg::Wasm(WasmMsg::Execute {
                contract_addr: VOTING_TOKEN.to_string(),
                msg: exec_msg_bz3,
                funds: vec![],
            })),
        ]
    );
    assert_eq!(
        execute_res.attributes,
        vec![attr("action", "execute_poll"), attr("poll_id", "1")]
    );

    // Query executed polls
    let res = query(
        deps.as_ref(),
        mock_env(),
        QueryMsg::Polls {
            filter: Some(PollStatus::Passed),
            start_after: None,
            limit: None,
            order_by: None,
        },
    )
    .unwrap();
    let response: PollsResponse = from_binary(&res).unwrap();
    assert_eq!(response.polls.len(), 0);

    let res = query(
        deps.as_ref(),
        mock_env(),
        QueryMsg::Polls {
            filter: Some(PollStatus::InProgress),
            start_after: None,
            limit: None,
            order_by: None,
        },
    )
    .unwrap();
    let response: PollsResponse = from_binary(&res).unwrap();
    assert_eq!(response.polls.len(), 0);

    let res = query(
        deps.as_ref(),
        mock_env(),
        QueryMsg::Polls {
            filter: Some(PollStatus::Executed),
            start_after: None,
            limit: None,
            order_by: Some(OrderBy::Desc),
        },
    )
    .unwrap();
    let response: PollsResponse = from_binary(&res).unwrap();
    assert_eq!(response.polls.len(), 1);

    // voter info must be deleted
    let res = query(
        deps.as_ref(),
        mock_env(),
        QueryMsg::Voters {
            poll_id: 1u64,
            start_after: None,
            limit: None,
            order_by: None,
        },
    )
    .unwrap();
    let response: VotersResponse = from_binary(&res).unwrap();
    assert_eq!(response.voters.len(), 0);

    // staker locked token must be disappeared
    let res = query(
        deps.as_ref(),
        mock_env(),
        QueryMsg::Staker {
            address: TEST_VOTER.to_string(),
        },
    )
    .unwrap();
    let response: StakerResponse = from_binary(&res).unwrap();
    assert_eq!(
        response,
        StakerResponse {
            balance: Uint128::from(stake_amount),
            share: Uint128::from(stake_amount),
            locked_balance: vec![],
        }
    );

    // But the data is still in the store
    let voter_addr_raw = deps.api.addr_canonicalize(TEST_VOTER).unwrap();
    let voter = poll_voter_read(&deps.storage, 1u64)
        .load(&voter_addr_raw.as_slice())
        .unwrap();
    assert_eq!(
        voter,
        VoterInfo {
            vote: VoteOption::Yes,
            balance: Uint128::from(stake_amount),
        }
    );

    let token_manager = bank_read(&deps.storage)
        .load(&voter_addr_raw.as_slice())
        .unwrap();
    assert_eq!(
        token_manager.locked_balance,
        vec![(
            1u64,
            VoterInfo {
                vote: VoteOption::Yes,
                balance: Uint128::from(stake_amount),
            }
        )]
    );
}

#[test]
fn end_poll_zero_quorum() {
    let mut deps = mock_dependencies(&coins(1000, VOTING_TOKEN));
    instantiate::mock_instantiate(deps.as_mut());
    mock_register_voting_token(deps.as_mut());
    let mut creator_env = common::mock_env_height(1000, 10000);
    let mut creator_info = mock_info(VOTING_TOKEN, &[]);

    let execute_msgs: Vec<PollExecuteMsg> = vec![PollExecuteMsg {
        order: 1u64,
        contract: VOTING_TOKEN.to_string(),
        msg: to_binary(&Cw20ExecuteMsg::Burn {
            amount: Uint128::new(123),
        })
        .unwrap(),
    }];

    let msg = create_poll_msg(
        "test".to_string(),
        "test".to_string(),
        None,
        Some(execute_msgs),
    );

    let execute_res = execute(
        deps.as_mut(),
        creator_env.clone(),
        creator_info.clone(),
        msg,
    )
    .unwrap();
    assert_create_poll_result(
        1,
        creator_env.block.height + DEFAULT_VOTING_PERIOD,
        TEST_CREATOR,
        execute_res,
        deps.as_ref(),
    );
    let stake_amount = 100;
    deps.querier.with_token_balances(&[(
        &VOTING_TOKEN.to_string(),
        &[(
            &MOCK_CONTRACT_ADDR.to_string(),
            &Uint128::from(100u128 + DEFAULT_PROPOSAL_DEPOSIT),
        )],
    )]);

    let msg = ExecuteMsg::Receive(Cw20ReceiveMsg {
        sender: TEST_VOTER.to_string(),
        amount: Uint128::from(stake_amount as u128),
        msg: to_binary(&Cw20HookMsg::StakeVotingTokens {}).unwrap(),
    });

    let info = mock_info(VOTING_TOKEN, &[]);
    execute(deps.as_mut(), mock_env(), info, msg).unwrap();

    let msg = ExecuteMsg::EndPoll { poll_id: 1 };
    creator_info.sender = Addr::unchecked(TEST_CREATOR);
    creator_env.block.height += DEFAULT_VOTING_PERIOD;

    let execute_res = execute(deps.as_mut(), creator_env, creator_info, msg).unwrap();

    assert_eq!(
        execute_res.attributes,
        vec![
            attr("action", "end_poll"),
            attr("poll_id", "1"),
            attr("rejected_reason", "Quorum not reached"),
            attr("passed", "false"),
        ]
    );

    assert_eq!(execute_res.messages.len(), 0usize);

    // Query rejected polls
    let res = query(
        deps.as_ref(),
        mock_env(),
        QueryMsg::Polls {
            filter: Some(PollStatus::Rejected),
            start_after: None,
            limit: None,
            order_by: Some(OrderBy::Desc),
        },
    )
    .unwrap();
    let response: PollsResponse = from_binary(&res).unwrap();
    assert_eq!(response.polls.len(), 1);

    let res = query(
        deps.as_ref(),
        mock_env(),
        QueryMsg::Polls {
            filter: Some(PollStatus::InProgress),
            start_after: None,
            limit: None,
            order_by: None,
        },
    )
    .unwrap();
    let response: PollsResponse = from_binary(&res).unwrap();
    assert_eq!(response.polls.len(), 0);

    let res = query(
        deps.as_ref(),
        mock_env(),
        QueryMsg::Polls {
            filter: Some(PollStatus::Passed),
            start_after: None,
            limit: None,
            order_by: None,
        },
    )
    .unwrap();
    let response: PollsResponse = from_binary(&res).unwrap();
    assert_eq!(response.polls.len(), 0);
}

#[test]
fn end_poll_quorum_rejected() {
    let mut deps = mock_dependencies(&coins(100, VOTING_TOKEN));
    instantiate::mock_instantiate(deps.as_mut());
    mock_register_voting_token(deps.as_mut());

    let msg = create_poll_msg("test".to_string(), "test".to_string(), None, None);
    let mut creator_env = mock_env();
    let mut creator_info = mock_info(VOTING_TOKEN, &[]);
    let execute_res = execute(
        deps.as_mut(),
        creator_env.clone(),
        creator_info.clone(),
        msg,
    )
    .unwrap();
    assert_eq!(
        execute_res.attributes,
        vec![
            attr("action", "create_poll"),
            attr("creator", TEST_CREATOR),
            attr("poll_id", "1"),
            attr("end_height", "22345"),
        ]
    );

    let stake_amount = 100;
    deps.querier.with_token_balances(&[(
        &VOTING_TOKEN.to_string(),
        &[(
            &MOCK_CONTRACT_ADDR.to_string(),
            &Uint128::from(100u128 + DEFAULT_PROPOSAL_DEPOSIT),
        )],
    )]);

    let msg = ExecuteMsg::Receive(Cw20ReceiveMsg {
        sender: TEST_VOTER.to_string(),
        amount: Uint128::from(stake_amount as u128),
        msg: to_binary(&Cw20HookMsg::StakeVotingTokens {}).unwrap(),
    });

    let info = mock_info(VOTING_TOKEN, &[]);
    let execute_res = execute(deps.as_mut(), mock_env(), info, msg).unwrap();
    assert_stake_tokens_result(
        stake_amount,
        DEFAULT_PROPOSAL_DEPOSIT,
        stake_amount,
        1,
        execute_res,
        deps.as_ref(),
    );

    let msg = ExecuteMsg::CastVote {
        poll_id: 1,
        vote: VoteOption::Yes,
        amount: Uint128::from(10u128),
    };
    let info = mock_info(TEST_VOTER, &[]);
    let execute_res = execute(deps.as_mut(), mock_env(), info, msg).unwrap();

    assert_eq!(
        execute_res.attributes,
        vec![
            attr("action", "cast_vote"),
            attr("poll_id", "1"),
            attr("amount", "10"),
            attr("voter", TEST_VOTER),
            attr("vote_option", "yes"),
        ]
    );

    let msg = ExecuteMsg::EndPoll { poll_id: 1 };

    creator_info.sender = Addr::unchecked(TEST_CREATOR);
    creator_env.block.height += DEFAULT_VOTING_PERIOD;

    let execute_res = execute(deps.as_mut(), creator_env, creator_info, msg).unwrap();
    assert_eq!(
        execute_res.attributes,
        vec![
            attr("action", "end_poll"),
            attr("poll_id", "1"),
            attr("rejected_reason", "Quorum not reached"),
            attr("passed", "false"),
        ]
    );
}

#[test]
fn end_poll_quorum_rejected_noting_staked() {
    let mut deps = mock_dependencies(&coins(100, VOTING_TOKEN));
    instantiate::mock_instantiate(deps.as_mut());
    mock_register_voting_token(deps.as_mut());

    let msg = create_poll_msg("test".to_string(), "test".to_string(), None, None);
    let mut creator_env = mock_env();
    let mut creator_info = mock_info(VOTING_TOKEN, &[]);
    let execute_res = execute(
        deps.as_mut(),
        creator_env.clone(),
        creator_info.clone(),
        msg,
    )
    .unwrap();
    assert_eq!(
        execute_res.attributes,
        vec![
            attr("action", "create_poll"),
            attr("creator", TEST_CREATOR),
            attr("poll_id", "1"),
            attr("end_height", "22345"),
        ]
    );

    let msg = ExecuteMsg::EndPoll { poll_id: 1 };

    creator_info.sender = Addr::unchecked(TEST_CREATOR);
    creator_env.block.height += DEFAULT_VOTING_PERIOD;

    let execute_res = execute(deps.as_mut(), creator_env, creator_info, msg).unwrap();
    assert_eq!(
        execute_res.attributes,
        vec![
            attr("action", "end_poll"),
            attr("poll_id", "1"),
            attr("rejected_reason", "Quorum not reached"),
            attr("passed", "false"),
        ]
    );
}

#[test]
fn end_poll_nay_rejected() {
    let voter1_stake = 100;
    let voter2_stake = 1000;
    let mut deps = mock_dependencies(&[]);
    instantiate::mock_instantiate(deps.as_mut());
    mock_register_voting_token(deps.as_mut());
    let mut creator_env = mock_env();
    let mut creator_info = mock_info(VOTING_TOKEN, &coins(2, VOTING_TOKEN));

    let msg = create_poll_msg("test".to_string(), "test".to_string(), None, None);

    let execute_res = execute(
        deps.as_mut(),
        creator_env.clone(),
        creator_info.clone(),
        msg,
    )
    .unwrap();
    assert_eq!(
        execute_res.attributes,
        vec![
            attr("action", "create_poll"),
            attr("creator", TEST_CREATOR),
            attr("poll_id", "1"),
            attr("end_height", "22345"),
        ]
    );

    deps.querier.with_token_balances(&[(
        &VOTING_TOKEN.to_string(),
        &[(
            &MOCK_CONTRACT_ADDR.to_string(),
            &Uint128::from((voter1_stake + DEFAULT_PROPOSAL_DEPOSIT) as u128),
        )],
    )]);

    let msg = ExecuteMsg::Receive(Cw20ReceiveMsg {
        sender: TEST_VOTER.to_string(),
        amount: Uint128::from(voter1_stake as u128),
        msg: to_binary(&Cw20HookMsg::StakeVotingTokens {}).unwrap(),
    });

    let info = mock_info(VOTING_TOKEN, &[]);
    let execute_res = execute(deps.as_mut(), mock_env(), info, msg).unwrap();
    assert_stake_tokens_result(
        voter1_stake,
        DEFAULT_PROPOSAL_DEPOSIT,
        voter1_stake,
        1,
        execute_res,
        deps.as_ref(),
    );

    deps.querier.with_token_balances(&[(
        &VOTING_TOKEN.to_string(),
        &[(
            &MOCK_CONTRACT_ADDR.to_string(),
            &Uint128::from((voter1_stake + voter2_stake + DEFAULT_PROPOSAL_DEPOSIT) as u128),
        )],
    )]);

    let msg = ExecuteMsg::Receive(Cw20ReceiveMsg {
        sender: TEST_VOTER_2.to_string(),
        amount: Uint128::from(voter2_stake as u128),
        msg: to_binary(&Cw20HookMsg::StakeVotingTokens {}).unwrap(),
    });

    let info = mock_info(VOTING_TOKEN, &[]);
    let execute_res = execute(deps.as_mut(), mock_env(), info, msg).unwrap();
    assert_stake_tokens_result(
        voter1_stake + voter2_stake,
        DEFAULT_PROPOSAL_DEPOSIT,
        voter2_stake,
        1,
        execute_res,
        deps.as_ref(),
    );

    let info = mock_info(TEST_VOTER_2, &[]);
    let msg = ExecuteMsg::CastVote {
        poll_id: 1,
        vote: VoteOption::No,
        amount: Uint128::from(voter2_stake),
    };
    let execute_res = execute(deps.as_mut(), mock_env(), info, msg).unwrap();
    assert_cast_vote_success(TEST_VOTER_2, voter2_stake, 1, VoteOption::No, execute_res);

    let msg = ExecuteMsg::EndPoll { poll_id: 1 };

    creator_info.sender = Addr::unchecked(TEST_CREATOR);
    creator_env.block.height += DEFAULT_VOTING_PERIOD;
    let execute_res = execute(deps.as_mut(), creator_env, creator_info, msg).unwrap();
    assert_eq!(
        execute_res.attributes,
        vec![
            attr("action", "end_poll"),
            attr("poll_id", "1"),
            attr("rejected_reason", "Threshold not reached"),
            attr("passed", "false"),
        ]
    );
}

#[test]
fn execute_poll_with_order() {
    const POLL_START_HEIGHT: u64 = 1000;
    const POLL_ID: u64 = 1;
    let stake_amount = 1000;

    let mut deps = mock_dependencies(&coins(1000, VOTING_TOKEN));
    instantiate::mock_instantiate(deps.as_mut());
    mock_register_voting_token(deps.as_mut());
    let mut creator_env = common::mock_env_height(POLL_START_HEIGHT, 10000);
    let mut creator_info = mock_info(VOTING_TOKEN, &coins(2, VOTING_TOKEN));

    let exec_msg_bz = to_binary(&Cw20ExecuteMsg::Burn {
        amount: Uint128::new(10),
    })
    .unwrap();

    let exec_msg_bz2 = to_binary(&Cw20ExecuteMsg::Burn {
        amount: Uint128::new(20),
    })
    .unwrap();

    let exec_msg_bz3 = to_binary(&Cw20ExecuteMsg::Burn {
        amount: Uint128::new(30),
    })
    .unwrap();
    let exec_msg_bz4 = to_binary(&Cw20ExecuteMsg::Burn {
        amount: Uint128::new(40),
    })
    .unwrap();
    let exec_msg_bz5 = to_binary(&Cw20ExecuteMsg::Burn {
        amount: Uint128::new(50),
    })
    .unwrap();

    //add three messages with different order
    let execute_msgs: Vec<PollExecuteMsg> = vec![
        PollExecuteMsg {
            order: 3u64,
            contract: VOTING_TOKEN.to_string(),
            msg: exec_msg_bz3.clone(),
        },
        PollExecuteMsg {
            order: 4u64,
            contract: VOTING_TOKEN.to_string(),
            msg: exec_msg_bz4.clone(),
        },
        PollExecuteMsg {
            order: 2u64,
            contract: VOTING_TOKEN.to_string(),
            msg: exec_msg_bz2.clone(),
        },
        PollExecuteMsg {
            order: 5u64,
            contract: VOTING_TOKEN.to_string(),
            msg: exec_msg_bz5.clone(),
        },
        PollExecuteMsg {
            order: 1u64,
            contract: VOTING_TOKEN.to_string(),
            msg: exec_msg_bz.clone(),
        },
    ];

    let msg = create_poll_msg(
        "test".to_string(),
        "test".to_string(),
        None,
        Some(execute_msgs),
    );

    let execute_res = execute(
        deps.as_mut(),
        creator_env.clone(),
        creator_info.clone(),
        msg,
    )
    .unwrap();

    assert_create_poll_result(
        1,
        creator_env.block.height + DEFAULT_VOTING_PERIOD,
        TEST_CREATOR,
        execute_res,
        deps.as_ref(),
    );

    deps.querier.with_token_balances(&[(
        &VOTING_TOKEN.to_string(),
        &[(
            &MOCK_CONTRACT_ADDR.to_string(),
            &Uint128::from((stake_amount + DEFAULT_PROPOSAL_DEPOSIT) as u128),
        )],
    )]);

    let msg = ExecuteMsg::Receive(Cw20ReceiveMsg {
        sender: TEST_VOTER.to_string(),
        amount: Uint128::from(stake_amount as u128),
        msg: to_binary(&Cw20HookMsg::StakeVotingTokens {}).unwrap(),
    });

    let info = mock_info(VOTING_TOKEN, &[]);
    let execute_res = execute(deps.as_mut(), mock_env(), info, msg).unwrap();
    assert_stake_tokens_result(
        stake_amount,
        DEFAULT_PROPOSAL_DEPOSIT,
        stake_amount,
        1,
        execute_res,
        deps.as_ref(),
    );

    let msg = ExecuteMsg::CastVote {
        poll_id: 1,
        vote: VoteOption::Yes,
        amount: Uint128::from(stake_amount),
    };
    let env = common::mock_env_height(POLL_START_HEIGHT, 10000);
    let info = mock_info(TEST_VOTER, &[]);
    let execute_res = execute(deps.as_mut(), env, info, msg).unwrap();

    assert_eq!(
        execute_res.attributes,
        vec![
            attr("action", "cast_vote"),
            attr("poll_id", POLL_ID.to_string().as_str()),
            attr("amount", "1000"),
            attr("voter", TEST_VOTER),
            attr("vote_option", "yes"),
        ]
    );

    creator_info.sender = Addr::unchecked(TEST_CREATOR);
    creator_env.block.height += DEFAULT_VOTING_PERIOD;

    let msg = ExecuteMsg::EndPoll { poll_id: 1 };
    let execute_res = execute(
        deps.as_mut(),
        creator_env.clone(),
        creator_info.clone(),
        msg,
    )
    .unwrap();

    assert_eq!(
        execute_res.attributes,
        vec![
            attr("action", "end_poll"),
            attr("poll_id", "1"),
            attr("rejected_reason", "Poll Passed"),
            attr("passed", "true"),
        ]
    );
    assert_eq!(
        execute_res.messages,
        vec![SubMsg::new(CosmosMsg::Wasm(WasmMsg::Execute {
            contract_addr: VOTING_TOKEN.to_string(),
            msg: to_binary(&Cw20ExecuteMsg::Transfer {
                recipient: TEST_CREATOR.to_string(),
                amount: Uint128::from(DEFAULT_PROPOSAL_DEPOSIT),
            })
            .unwrap(),
            funds: vec![],
        }))]
    );

    // End poll will withdraw deposit balance
    deps.querier.with_token_balances(&[(
        &VOTING_TOKEN.to_string(),
        &[(
            &MOCK_CONTRACT_ADDR.to_string(),
            &Uint128::from(stake_amount as u128),
        )],
    )]);

    creator_env.block.height += DEFAULT_TIMELOCK_PERIOD;
    let msg = ExecuteMsg::ExecutePoll { poll_id: 1 };
    let execute_res = execute(deps.as_mut(), creator_env, creator_info, msg).unwrap();
    assert_eq!(
        execute_res.messages,
        vec![
            SubMsg::new(CosmosMsg::Wasm(WasmMsg::Execute {
                contract_addr: VOTING_TOKEN.to_string(),
                msg: exec_msg_bz,
                funds: vec![],
            })),
            SubMsg::new(CosmosMsg::Wasm(WasmMsg::Execute {
                contract_addr: VOTING_TOKEN.to_string(),
                msg: exec_msg_bz2,
                funds: vec![],
            })),
            SubMsg::new(CosmosMsg::Wasm(WasmMsg::Execute {
                contract_addr: VOTING_TOKEN.to_string(),
                msg: exec_msg_bz3,
                funds: vec![],
            })),
            SubMsg::new(CosmosMsg::Wasm(WasmMsg::Execute {
                contract_addr: VOTING_TOKEN.to_string(),
                msg: exec_msg_bz4,
                funds: vec![],
            })),
            SubMsg::new(CosmosMsg::Wasm(WasmMsg::Execute {
                contract_addr: VOTING_TOKEN.to_string(),
                msg: exec_msg_bz5,
                funds: vec![],
            })),
        ]
    );
    assert_eq!(
        execute_res.attributes,
        vec![attr("action", "execute_poll"), attr("poll_id", "1")]
    );
}

#[test]
fn snapshot_poll() {
    let stake_amount = 1000;

    let mut deps = mock_dependencies(&coins(100, VOTING_TOKEN));
    instantiate::mock_instantiate(deps.as_mut());
    mock_register_voting_token(deps.as_mut());

    let msg = create_poll_msg("test".to_string(), "test".to_string(), None, None);
    let mut creator_env = mock_env();
    let creator_info = mock_info(VOTING_TOKEN, &[]);
    let execute_res = execute(
        deps.as_mut(),
        creator_env.clone(),
        creator_info.clone(),
        msg,
    )
    .unwrap();
    assert_eq!(
        execute_res.attributes,
        vec![
            attr("action", "create_poll"),
            attr("creator", TEST_CREATOR),
            attr("poll_id", "1"),
            attr("end_height", "22345"),
        ]
    );

    //must not be executed
    let snapshot_err = execute(
        deps.as_mut(),
        creator_env.clone(),
        creator_info.clone(),
        ExecuteMsg::SnapshotPoll { poll_id: 1 },
    )
    .unwrap_err();
    assert_eq!(ContractError::SnapshotHeight {}, snapshot_err);

    // change time
    creator_env.block.height = 22345 - 10;

    deps.querier.with_token_balances(&[(
        &VOTING_TOKEN.to_string(),
        &[(
            &MOCK_CONTRACT_ADDR.to_string(),
            &Uint128::from((stake_amount + DEFAULT_PROPOSAL_DEPOSIT) as u128),
        )],
    )]);

    let fix_res = execute(
        deps.as_mut(),
        creator_env.clone(),
        creator_info.clone(),
        ExecuteMsg::SnapshotPoll { poll_id: 1 },
    )
    .unwrap();

    assert_eq!(
        fix_res.attributes,
        vec![
            attr("action", "snapshot_poll"),
            attr("poll_id", "1"),
            attr("staked_amount", stake_amount.to_string().as_str()),
        ]
    );

    //must not be executed
    let snapshot_error = execute(
        deps.as_mut(),
        creator_env,
        creator_info,
        ExecuteMsg::SnapshotPoll { poll_id: 1 },
    )
    .unwrap_err();
    assert_eq!(ContractError::SnapshotAlreadyOccurred {}, snapshot_error);
}

#[test]
fn successful_cast_vote_with_snapshot() {
    let mut deps = mock_dependencies(&[]);
    instantiate::mock_instantiate(deps.as_mut());
    mock_register_voting_token(deps.as_mut());

    let env = common::mock_env_height(0, 10000);
    let info = mock_info(VOTING_TOKEN, &[]);
    let msg = create_poll_msg("test".to_string(), "test".to_string(), None, None);

    let execute_res = execute(deps.as_mut(), env, info, msg).unwrap();
    assert_create_poll_result(
        1,
        DEFAULT_VOTING_PERIOD,
        TEST_CREATOR,
        execute_res,
        deps.as_ref(),
    );

    deps.querier.with_token_balances(&[(
        &VOTING_TOKEN.to_string(),
        &[(
            &MOCK_CONTRACT_ADDR.to_string(),
            &Uint128::from(11u128 + DEFAULT_PROPOSAL_DEPOSIT),
        )],
    )]);

    let msg = ExecuteMsg::Receive(Cw20ReceiveMsg {
        sender: TEST_VOTER.to_string(),
        amount: Uint128::from(11u128),
        msg: to_binary(&Cw20HookMsg::StakeVotingTokens {}).unwrap(),
    });

    let info = mock_info(VOTING_TOKEN, &[]);
    let execute_res = execute(deps.as_mut(), mock_env(), info, msg).unwrap();
    assert_stake_tokens_result(
        11,
        DEFAULT_PROPOSAL_DEPOSIT,
        11,
        1,
        execute_res,
        deps.as_ref(),
    );

    //cast_vote without snapshot
    let env = common::mock_env_height(0, 10000);
    let info = mock_info(TEST_VOTER, &coins(11, VOTING_TOKEN));
    let amount = 10u128;

    let msg = ExecuteMsg::CastVote {
        poll_id: 1,
        vote: VoteOption::Yes,
        amount: Uint128::from(amount),
    };

    let execute_res = execute(deps.as_mut(), env, info, msg).unwrap();
    assert_cast_vote_success(TEST_VOTER, amount, 1, VoteOption::Yes, execute_res);

    // balance be double
    deps.querier.with_token_balances(&[(
        &VOTING_TOKEN.to_string(),
        &[(
            &MOCK_CONTRACT_ADDR.to_string(),
            &Uint128::from(22u128 + DEFAULT_PROPOSAL_DEPOSIT),
        )],
    )]);

    let res = query(deps.as_ref(), mock_env(), QueryMsg::Poll { poll_id: 1 }).unwrap();
    let value: PollResponse = from_binary(&res).unwrap();
    assert_eq!(value.staked_amount, None);
    let end_height = value.end_height;

    //cast another vote
    let msg = ExecuteMsg::Receive(Cw20ReceiveMsg {
        sender: TEST_VOTER_2.to_string(),
        amount: Uint128::from(11u128),
        msg: to_binary(&Cw20HookMsg::StakeVotingTokens {}).unwrap(),
    });

    let info = mock_info(VOTING_TOKEN, &[]);
    let _execute_res = execute(deps.as_mut(), mock_env(), info, msg).unwrap();

    // another voter cast a vote
    let msg = ExecuteMsg::CastVote {
        poll_id: 1,
        vote: VoteOption::Yes,
        amount: Uint128::from(10u128),
    };
    let env = common::mock_env_height(end_height - 9, 10000);
    let info = mock_info(TEST_VOTER_2, &[]);
    let execute_res = execute(deps.as_mut(), env.clone(), info.clone(), msg).unwrap();
    assert_cast_vote_success(TEST_VOTER_2, amount, 1, VoteOption::Yes, execute_res);

    let res = query(deps.as_ref(), mock_env(), QueryMsg::Poll { poll_id: 1 }).unwrap();
    let value: PollResponse = from_binary(&res).unwrap();
    assert_eq!(value.staked_amount, Some(Uint128::new(22)));

    // snanpshot poll will not go through
    let snap_error = execute(
        deps.as_mut(),
        env,
        info,
        ExecuteMsg::SnapshotPoll { poll_id: 1 },
    )
    .unwrap_err();
    assert_eq!(ContractError::SnapshotAlreadyOccurred {}, snap_error);

    // balance be double
    deps.querier.with_token_balances(&[(
        &VOTING_TOKEN.to_string(),
        &[(
            &MOCK_CONTRACT_ADDR.to_string(),
            &Uint128::from(33u128 + DEFAULT_PROPOSAL_DEPOSIT),
        )],
    )]);

    // another voter cast a vote but the snapshot is already occurred
    let msg = ExecuteMsg::Receive(Cw20ReceiveMsg {
        sender: TEST_VOTER_3.to_string(),
        amount: Uint128::from(11u128),
        msg: to_binary(&Cw20HookMsg::StakeVotingTokens {}).unwrap(),
    });

    let info = mock_info(VOTING_TOKEN, &[]);
    let _execute_res = execute(deps.as_mut(), mock_env(), info, msg).unwrap();
    let msg = ExecuteMsg::CastVote {
        poll_id: 1,
        vote: VoteOption::Yes,
        amount: Uint128::from(10u128),
    };
    let env = common::mock_env_height(end_height - 8, 10000);
    let info = mock_info(TEST_VOTER_3, &[]);
    let execute_res = execute(deps.as_mut(), env, info, msg).unwrap();
    assert_cast_vote_success(TEST_VOTER_3, amount, 1, VoteOption::Yes, execute_res);

    let res = query(deps.as_ref(), mock_env(), QueryMsg::Poll { poll_id: 1 }).unwrap();
    let value: PollResponse = from_binary(&res).unwrap();
    assert_eq!(value.staked_amount, Some(Uint128::new(22)));
}

#[test]
fn fails_end_poll_quorum_inflation_without_snapshot_poll() {
    const POLL_START_HEIGHT: u64 = 1000;
    const POLL_ID: u64 = 1;
    let stake_amount = 1000;

    let mut deps = mock_dependencies(&coins(1000, VOTING_TOKEN));
    instantiate::mock_instantiate(deps.as_mut());
    mock_register_voting_token(deps.as_mut());

    let mut creator_env = common::mock_env_height(POLL_START_HEIGHT, 10000);
    let mut creator_info = mock_info(VOTING_TOKEN, &coins(2, VOTING_TOKEN));

    let exec_msg_bz = to_binary(&Cw20ExecuteMsg::Burn {
        amount: Uint128::new(123),
    })
    .unwrap();

    //add two messages
    let execute_msgs: Vec<PollExecuteMsg> = vec![
        PollExecuteMsg {
            order: 1u64,
            contract: VOTING_TOKEN.to_string(),
            msg: exec_msg_bz.clone(),
        },
        PollExecuteMsg {
            order: 2u64,
            contract: VOTING_TOKEN.to_string(),
            msg: exec_msg_bz,
        },
    ];

    let msg = create_poll_msg(
        "test".to_string(),
        "test".to_string(),
        None,
        Some(execute_msgs),
    );

    let execute_res = execute(
        deps.as_mut(),
        creator_env.clone(),
        creator_info.clone(),
        msg,
    )
    .unwrap();

    assert_create_poll_result(
        1,
        creator_env.block.height + DEFAULT_VOTING_PERIOD,
        TEST_CREATOR,
        execute_res,
        deps.as_ref(),
    );

    deps.querier.with_token_balances(&[(
        &VOTING_TOKEN.to_string(),
        &[(
            &MOCK_CONTRACT_ADDR.to_string(),
            &Uint128::from((stake_amount + DEFAULT_PROPOSAL_DEPOSIT) as u128),
        )],
    )]);

    let msg = ExecuteMsg::Receive(Cw20ReceiveMsg {
        sender: TEST_VOTER.to_string(),
        amount: Uint128::from(stake_amount as u128),
        msg: to_binary(&Cw20HookMsg::StakeVotingTokens {}).unwrap(),
    });

    let info = mock_info(VOTING_TOKEN, &[]);
    let execute_res = execute(deps.as_mut(), mock_env(), info, msg).unwrap();
    assert_stake_tokens_result(
        stake_amount,
        DEFAULT_PROPOSAL_DEPOSIT,
        stake_amount,
        1,
        execute_res,
        deps.as_ref(),
    );

    let msg = ExecuteMsg::CastVote {
        poll_id: 1,
        vote: VoteOption::Yes,
        amount: Uint128::from(stake_amount),
    };
    let env = common::mock_env_height(POLL_START_HEIGHT, 10000);
    let info = mock_info(TEST_VOTER, &[]);
    let execute_res = execute(deps.as_mut(), env, info, msg).unwrap();

    assert_eq!(
        execute_res.attributes,
        vec![
            attr("action", "cast_vote"),
            attr("poll_id", POLL_ID.to_string().as_str()),
            attr("amount", "1000"),
            attr("voter", TEST_VOTER),
            attr("vote_option", "yes"),
        ]
    );

    creator_env.block.height += DEFAULT_VOTING_PERIOD - 10;

    // did not SnapshotPoll

    // staked amount get increased 10 times
    deps.querier.with_token_balances(&[(
        &VOTING_TOKEN.to_string(),
        &[(
            &MOCK_CONTRACT_ADDR.to_string(),
            &Uint128::from(((10 * stake_amount) + DEFAULT_PROPOSAL_DEPOSIT) as u128),
        )],
    )]);

    //cast another vote
    let msg = ExecuteMsg::Receive(Cw20ReceiveMsg {
        sender: TEST_VOTER_2.to_string(),
        amount: Uint128::from(8 * stake_amount as u128),
        msg: to_binary(&Cw20HookMsg::StakeVotingTokens {}).unwrap(),
    });

    let info = mock_info(VOTING_TOKEN, &[]);
    let _execute_res = execute(deps.as_mut(), mock_env(), info, msg).unwrap();

    // another voter cast a vote
    let msg = ExecuteMsg::CastVote {
        poll_id: 1,
        vote: VoteOption::Yes,
        amount: Uint128::from(stake_amount),
    };
    let env = common::mock_env_height(creator_env.block.height, 10000);
    let info = mock_info(TEST_VOTER_2, &[]);
    let execute_res = execute(deps.as_mut(), env, info, msg).unwrap();

    assert_eq!(
        execute_res.attributes,
        vec![
            attr("action", "cast_vote"),
            attr("poll_id", POLL_ID.to_string().as_str()),
            attr("amount", "1000"),
            attr("voter", TEST_VOTER_2),
            attr("vote_option", "yes"),
        ]
    );

    creator_info.sender = Addr::unchecked(TEST_CREATOR);
    creator_env.block.height += 10;

    // quorum must reach
    let msg = ExecuteMsg::EndPoll { poll_id: 1 };
    let execute_res = execute(deps.as_mut(), creator_env, creator_info, msg).unwrap();

    assert_eq!(
        execute_res.attributes,
        vec![
            attr("action", "end_poll"),
            attr("poll_id", "1"),
            attr("rejected_reason", "Quorum not reached"),
            attr("passed", "false"),
        ]
    );

    let res = query(deps.as_ref(), mock_env(), QueryMsg::Poll { poll_id: 1 }).unwrap();
    let value: PollResponse = from_binary(&res).unwrap();
    assert_eq!(
        10 * stake_amount,
        value.total_balance_at_end_poll.unwrap().u128()
    );
}

#[test]
fn successful_end_poll_with_controlled_quorum() {
    const POLL_START_HEIGHT: u64 = 1000;
    const POLL_ID: u64 = 1;
    let stake_amount = 1000;

    let mut deps = mock_dependencies(&coins(1000, VOTING_TOKEN));
    instantiate::mock_instantiate(deps.as_mut());
    mock_register_voting_token(deps.as_mut());

    let mut creator_env = common::mock_env_height(POLL_START_HEIGHT, 10000);
    let mut creator_info = mock_info(VOTING_TOKEN, &coins(2, VOTING_TOKEN));

    let exec_msg_bz = to_binary(&Cw20ExecuteMsg::Burn {
        amount: Uint128::new(123),
    })
    .unwrap();

    //add two messages
    let execute_msgs: Vec<PollExecuteMsg> = vec![
        PollExecuteMsg {
            order: 1u64,
            contract: VOTING_TOKEN.to_string(),
            msg: exec_msg_bz.clone(),
        },
        PollExecuteMsg {
            order: 2u64,
            contract: VOTING_TOKEN.to_string(),
            msg: exec_msg_bz,
        },
    ];

    let msg = create_poll_msg(
        "test".to_string(),
        "test".to_string(),
        None,
        Some(execute_msgs),
    );

    let execute_res = execute(
        deps.as_mut(),
        creator_env.clone(),
        creator_info.clone(),
        msg,
    )
    .unwrap();

    assert_create_poll_result(
        1,
        creator_env.block.height + DEFAULT_VOTING_PERIOD,
        TEST_CREATOR,
        execute_res,
        deps.as_ref(),
    );

    deps.querier.with_token_balances(&[(
        &VOTING_TOKEN.to_string(),
        &[(
            &MOCK_CONTRACT_ADDR.to_string(),
            &Uint128::from((stake_amount + DEFAULT_PROPOSAL_DEPOSIT) as u128),
        )],
    )]);

    let msg = ExecuteMsg::Receive(Cw20ReceiveMsg {
        sender: TEST_VOTER.to_string(),
        amount: Uint128::from(stake_amount as u128),
        msg: to_binary(&Cw20HookMsg::StakeVotingTokens {}).unwrap(),
    });

    let info = mock_info(VOTING_TOKEN, &[]);
    let execute_res = execute(deps.as_mut(), mock_env(), info, msg).unwrap();
    assert_stake_tokens_result(
        stake_amount,
        DEFAULT_PROPOSAL_DEPOSIT,
        stake_amount,
        1,
        execute_res,
        deps.as_ref(),
    );

    let msg = ExecuteMsg::CastVote {
        poll_id: 1,
        vote: VoteOption::Yes,
        amount: Uint128::from(stake_amount),
    };
    let env = common::mock_env_height(POLL_START_HEIGHT, 10000);
    let info = mock_info(TEST_VOTER, &[]);
    let execute_res = execute(deps.as_mut(), env, info, msg).unwrap();

    assert_eq!(
        execute_res.attributes,
        vec![
            attr("action", "cast_vote"),
            attr("poll_id", POLL_ID.to_string().as_str()),
            attr("amount", "1000"),
            attr("voter", TEST_VOTER),
            attr("vote_option", "yes"),
        ]
    );

    creator_env.block.height += DEFAULT_VOTING_PERIOD - 10;

    // send SnapshotPoll
    let fix_res = execute(
        deps.as_mut(),
        creator_env.clone(),
        creator_info.clone(),
        ExecuteMsg::SnapshotPoll { poll_id: 1 },
    )
    .unwrap();

    assert_eq!(
        fix_res.attributes,
        vec![
            attr("action", "snapshot_poll"),
            attr("poll_id", "1"),
            attr("staked_amount", stake_amount.to_string().as_str()),
        ]
    );

    // staked amount get increased 10 times
    deps.querier.with_token_balances(&[(
        &VOTING_TOKEN.to_string(),
        &[(
            &MOCK_CONTRACT_ADDR.to_string(),
            &Uint128::from(((10 * stake_amount) + DEFAULT_PROPOSAL_DEPOSIT) as u128),
        )],
    )]);

    //cast another vote
    let msg = ExecuteMsg::Receive(Cw20ReceiveMsg {
        sender: TEST_VOTER_2.to_string(),
        amount: Uint128::from(8 * stake_amount as u128),
        msg: to_binary(&Cw20HookMsg::StakeVotingTokens {}).unwrap(),
    });

    let info = mock_info(VOTING_TOKEN, &[]);
    let _execute_res = execute(deps.as_mut(), mock_env(), info, msg).unwrap();

    let msg = ExecuteMsg::CastVote {
        poll_id: 1,
        vote: VoteOption::Yes,
        amount: Uint128::from(8 * stake_amount),
    };
    let env = common::mock_env_height(creator_env.block.height, 10000);
    let info = mock_info(TEST_VOTER_2, &[]);
    let execute_res = execute(deps.as_mut(), env, info, msg).unwrap();

    assert_eq!(
        execute_res.attributes,
        vec![
            attr("action", "cast_vote"),
            attr("poll_id", POLL_ID.to_string().as_str()),
            attr("amount", "8000"),
            attr("voter", TEST_VOTER_2),
            attr("vote_option", "yes"),
        ]
    );

    creator_info.sender = Addr::unchecked(TEST_CREATOR);
    creator_env.block.height += 10;

    // quorum must reach
    let msg = ExecuteMsg::EndPoll { poll_id: 1 };
    let execute_res = execute(deps.as_mut(), creator_env, creator_info, msg).unwrap();

    assert_eq!(
        execute_res.attributes,
        vec![
            attr("action", "end_poll"),
            attr("poll_id", "1"),
            attr("rejected_reason", "Poll Passed"),
            attr("passed", "true"),
        ]
    );
    assert_eq!(
        execute_res.messages,
        vec![SubMsg::new(CosmosMsg::Wasm(WasmMsg::Execute {
            contract_addr: VOTING_TOKEN.to_string(),
            msg: to_binary(&Cw20ExecuteMsg::Transfer {
                recipient: TEST_CREATOR.to_string(),
                amount: Uint128::from(DEFAULT_PROPOSAL_DEPOSIT),
            })
            .unwrap(),
            funds: vec![],
        }))]
    );

    let res = query(deps.as_ref(), mock_env(), QueryMsg::Poll { poll_id: 1 }).unwrap();
    let value: PollResponse = from_binary(&res).unwrap();
    assert_eq!(
        stake_amount,
        value.total_balance_at_end_poll.unwrap().u128()
    );

    assert_eq!(value.yes_votes.u128(), 9 * stake_amount);

    // actual staked amount is 10 times bigger than staked amount
    let actual_staked_weight = query_token_balance(
        &deps.as_ref().querier,
        Addr::unchecked(VOTING_TOKEN),
        Addr::unchecked(MOCK_CONTRACT_ADDR),
    )
    .unwrap()
    .checked_sub(Uint128::from(DEFAULT_PROPOSAL_DEPOSIT))
    .unwrap();

    assert_eq!(actual_staked_weight.u128(), (10 * stake_amount))
}

#[test]
fn fails_query_poll() {
    let mut deps = mock_dependencies(&[]);
    instantiate::mock_instantiate(deps.as_mut());
    mock_register_voting_token(deps.as_mut());
    let env = mock_env_height(0, 10000);
    let info = mock_info(VOTING_TOKEN, &[]);

    let msg = create_poll_msg("test".to_string(), "test".to_string(), None, None);

    execute(deps.as_mut(), env.clone(), info, msg).unwrap();

    match query(deps.as_ref(), mock_env(), QueryMsg::Poll { poll_id: 2 }) {
        Ok(_) => panic!("Must return error"),
        Err(ContractError::PollNotFound {}) => (),
        Err(_) => panic!("Unknown error"),
    };
}
