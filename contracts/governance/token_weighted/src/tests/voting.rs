use crate::contract::{execute, query};
use crate::{ExecuteMsg, QueryMsg};
use crate::state::{
    bank_read, bank_store, poll_store, poll_voter_read, poll_voter_store, state_read, Cw20HookMsg,
    OrderBy, Poll, PollStatus, StakerResponse, State, TokenManager, VoteOption, VoterInfo,
    VotersResponse, VotersResponseItem,
};
use crate::tests::common::{
    DEFAULT_PROPOSAL_DEPOSIT, DEFAULT_VOTING_PERIOD, TEST_CREATOR, TEST_VOTER, VOTING_TOKEN,
};
use crate::tests::mock_querier::mock_dependencies;
use crate::tests::{common, instantiate, poll};
use crate::ContractError;
use cosmwasm_std::testing::{mock_env, mock_info, MOCK_CONTRACT_ADDR};
use cosmwasm_std::{
    coins, from_binary, to_binary, Api, CanonicalAddr, CosmosMsg, SubMsg, Uint128, WasmMsg,
};
use cw20::{Cw20ExecuteMsg, Cw20ReceiveMsg};

#[test]
fn fails_cast_vote_not_enough_staked() {
    let mut deps = mock_dependencies(&[]);
    instantiate::mock_instantiate(deps.as_mut());
    poll::mock_register_voting_token(deps.as_mut());
    let env = common::mock_env_height(0, 10000);
    let info = mock_info(VOTING_TOKEN, &[]);

    let msg = poll::create_poll_msg("test".to_string(), "test".to_string(), None, None);

    let execute_res = execute(deps.as_mut(), env, info, msg).unwrap();
    poll::assert_create_poll_result(
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
            &Uint128::from(10u128 + DEFAULT_PROPOSAL_DEPOSIT),
        )],
    )]);

    let msg = ExecuteMsg::Receive(Cw20ReceiveMsg {
        sender: TEST_VOTER.to_string(),
        amount: Uint128::from(10u128),
        msg: to_binary(&Cw20HookMsg::StakeVotingTokens {}).unwrap(),
    });

    let info = mock_info(VOTING_TOKEN, &[]);
    let execute_res = execute(deps.as_mut(), mock_env(), info, msg).unwrap();
    poll::assert_stake_tokens_result(
        10,
        DEFAULT_PROPOSAL_DEPOSIT,
        10,
        1,
        execute_res,
        deps.as_ref(),
    );

    let env = common::mock_env_height(0, 10000);
    let info = mock_info(TEST_VOTER, &coins(11, VOTING_TOKEN));
    let msg = ExecuteMsg::CastVote {
        poll_id: 1,
        vote: VoteOption::Yes,
        amount: Uint128::from(11u128),
    };

    let res = execute(deps.as_mut(), env, info, msg);

    match res {
        Ok(_) => panic!("Must return error"),
        Err(ContractError::InsufficientStaked {}) => (),
        Err(e) => panic!("Unexpected error: {:?}", e),
    }
}

#[test]
fn successful_cast_vote() {
    let mut deps = mock_dependencies(&[]);
    instantiate::mock_instantiate(deps.as_mut());
    poll::mock_register_voting_token(deps.as_mut());

    let env = common::mock_env_height(0, 10000);
    let info = mock_info(VOTING_TOKEN, &[]);
    let msg = poll::create_poll_msg("test".to_string(), "test".to_string(), None, None);

    let execute_res = execute(deps.as_mut(), env, info, msg).unwrap();
    poll::assert_create_poll_result(
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
    poll::assert_stake_tokens_result(
        11,
        DEFAULT_PROPOSAL_DEPOSIT,
        11,
        1,
        execute_res,
        deps.as_ref(),
    );

    let env = common::mock_env_height(0, 10000);
    let info = mock_info(TEST_VOTER, &coins(11, VOTING_TOKEN));
    let amount = 10u128;
    let msg = ExecuteMsg::CastVote {
        poll_id: 1,
        vote: VoteOption::Yes,
        amount: Uint128::from(amount),
    };

    let execute_res = execute(deps.as_mut(), env, info, msg).unwrap();
    poll::assert_cast_vote_success(TEST_VOTER, amount, 1, VoteOption::Yes, execute_res);

    // balance be double
    deps.querier.with_token_balances(&[(
        &VOTING_TOKEN.to_string(),
        &[(
            &MOCK_CONTRACT_ADDR.to_string(),
            &Uint128::from(22u128 + DEFAULT_PROPOSAL_DEPOSIT),
        )],
    )]);

    // Query staker
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
            balance: Uint128::from(22u128),
            share: Uint128::from(11u128),
            locked_balance: vec![(
                1u64,
                VoterInfo {
                    vote: VoteOption::Yes,
                    balance: Uint128::from(amount),
                }
            )]
        }
    );

    // Query voters
    let res = query(
        deps.as_ref(),
        mock_env(),
        QueryMsg::Voters {
            poll_id: 1u64,
            start_after: None,
            limit: None,
            order_by: Some(OrderBy::Desc),
        },
    )
    .unwrap();
    let response: VotersResponse = from_binary(&res).unwrap();
    assert_eq!(
        response.voters,
        vec![VotersResponseItem {
            voter: TEST_VOTER.to_string(),
            vote: VoteOption::Yes,
            balance: Uint128::from(amount),
        }]
    );

    let res = query(
        deps.as_ref(),
        mock_env(),
        QueryMsg::Voters {
            poll_id: 1u64,
            start_after: Some(TEST_VOTER.to_string()),
            limit: None,
            order_by: None,
        },
    )
    .unwrap();
    let response: VotersResponse = from_binary(&res).unwrap();
    assert_eq!(response.voters.len(), 0);
}

#[test]
fn successful_withdraw_voting_tokens() {
    let mut deps = mock_dependencies(&[]);
    instantiate::mock_instantiate(deps.as_mut());
    poll::mock_register_voting_token(deps.as_mut());

    deps.querier.with_token_balances(&[(
        &VOTING_TOKEN.to_string(),
        &[(&MOCK_CONTRACT_ADDR.to_string(), &Uint128::from(11u128))],
    )]);

    let msg = ExecuteMsg::Receive(Cw20ReceiveMsg {
        sender: TEST_VOTER.to_string(),
        amount: Uint128::from(11u128),
        msg: to_binary(&Cw20HookMsg::StakeVotingTokens {}).unwrap(),
    });

    let info = mock_info(VOTING_TOKEN, &[]);
    let execute_res = execute(deps.as_mut(), mock_env(), info, msg).unwrap();
    poll::assert_stake_tokens_result(11, 0, 11, 0, execute_res, deps.as_ref());

    let state: State = state_read(deps.as_ref().storage).load().unwrap();
    assert_eq!(
        state,
        State {
            contract_addr: deps.api.addr_canonicalize(MOCK_CONTRACT_ADDR).unwrap(),
            poll_count: 0,
            total_share: Uint128::from(11u128),
            total_deposit: Uint128::zero(),
        }
    );

    // double the balance, only half will be withdrawn
    deps.querier.with_token_balances(&[(
        &VOTING_TOKEN.to_string(),
        &[(&MOCK_CONTRACT_ADDR.to_string(), &Uint128::from(22u128))],
    )]);

    let info = mock_info(TEST_VOTER, &[]);
    let msg = ExecuteMsg::WithdrawVotingTokens {
        amount: Some(Uint128::from(11u128)),
    };

    let execute_res = execute(deps.as_mut(), mock_env(), info, msg).unwrap();
    let msg = execute_res.messages.get(0).expect("no message");

    assert_eq!(
        msg,
        &SubMsg::new(CosmosMsg::Wasm(WasmMsg::Execute {
            contract_addr: VOTING_TOKEN.to_string(),
            msg: to_binary(&Cw20ExecuteMsg::Transfer {
                recipient: TEST_VOTER.to_string(),
                amount: Uint128::from(11u128),
            })
            .unwrap(),
            funds: vec![],
        }))
    );

    let state: State = state_read(deps.as_ref().storage).load().unwrap();
    assert_eq!(
        state,
        State {
            contract_addr: deps.api.addr_canonicalize(MOCK_CONTRACT_ADDR).unwrap(),
            poll_count: 0,
            total_share: Uint128::from(6u128),
            total_deposit: Uint128::zero(),
        }
    );
}

#[test]
fn successful_withdraw_voting_tokens_all() {
    let mut deps = mock_dependencies(&[]);
    instantiate::mock_instantiate(deps.as_mut());
    poll::mock_register_voting_token(deps.as_mut());

    deps.querier.with_token_balances(&[(
        &VOTING_TOKEN.to_string(),
        &[(&MOCK_CONTRACT_ADDR.to_string(), &Uint128::from(11u128))],
    )]);

    let msg = ExecuteMsg::Receive(Cw20ReceiveMsg {
        sender: TEST_VOTER.to_string(),
        amount: Uint128::from(11u128),
        msg: to_binary(&Cw20HookMsg::StakeVotingTokens {}).unwrap(),
    });

    let info = mock_info(VOTING_TOKEN, &[]);
    let execute_res = execute(deps.as_mut(), mock_env(), info, msg).unwrap();
    poll::assert_stake_tokens_result(11, 0, 11, 0, execute_res, deps.as_ref());

    let state: State = state_read(deps.as_ref().storage).load().unwrap();
    assert_eq!(
        state,
        State {
            contract_addr: deps.api.addr_canonicalize(MOCK_CONTRACT_ADDR).unwrap(),
            poll_count: 0,
            total_share: Uint128::from(11u128),
            total_deposit: Uint128::zero(),
        }
    );

    // double the balance, all balance withdrawn
    deps.querier.with_token_balances(&[(
        &VOTING_TOKEN.to_string(),
        &[(&MOCK_CONTRACT_ADDR.to_string(), &Uint128::from(22u128))],
    )]);

    let info = mock_info(TEST_VOTER, &[]);
    let msg = ExecuteMsg::WithdrawVotingTokens { amount: None };

    let execute_res = execute(deps.as_mut(), mock_env(), info, msg).unwrap();
    let msg = execute_res.messages.get(0).expect("no message");

    assert_eq!(
        msg,
        &SubMsg::new(CosmosMsg::Wasm(WasmMsg::Execute {
            contract_addr: VOTING_TOKEN.to_string(),
            msg: to_binary(&Cw20ExecuteMsg::Transfer {
                recipient: TEST_VOTER.to_string(),
                amount: Uint128::from(22u128),
            })
            .unwrap(),
            funds: vec![],
        }))
    );

    let state: State = state_read(deps.as_ref().storage).load().unwrap();
    assert_eq!(
        state,
        State {
            contract_addr: deps.api.addr_canonicalize(MOCK_CONTRACT_ADDR).unwrap(),
            poll_count: 0,
            total_share: Uint128::zero(),
            total_deposit: Uint128::zero(),
        }
    );
}

#[test]
fn withdraw_voting_tokens_remove_not_in_progress_poll_voter_info() {
    let mut deps = mock_dependencies(&[]);
    instantiate::mock_instantiate(deps.as_mut());
    poll::mock_register_voting_token(deps.as_mut());

    deps.querier.with_token_balances(&[(
        &VOTING_TOKEN.to_string(),
        &[(&MOCK_CONTRACT_ADDR.to_string(), &Uint128::from(11u128))],
    )]);

    let msg = ExecuteMsg::Receive(Cw20ReceiveMsg {
        sender: TEST_VOTER.to_string(),
        amount: Uint128::from(11u128),
        msg: to_binary(&Cw20HookMsg::StakeVotingTokens {}).unwrap(),
    });

    let info = mock_info(VOTING_TOKEN, &[]);
    let execute_res = execute(deps.as_mut(), mock_env(), info, msg).unwrap();
    poll::assert_stake_tokens_result(11, 0, 11, 0, execute_res, deps.as_ref());

    // make fake polls; one in progress & one in passed
    poll_store(&mut deps.storage)
        .save(
            &1u64.to_be_bytes(),
            &Poll {
                id: 1u64,
                creator: CanonicalAddr::from(vec![]),
                status: PollStatus::InProgress,
                yes_votes: Uint128::zero(),
                no_votes: Uint128::zero(),
                end_height: 0u64,
                title: "title".to_string(),
                description: "description".to_string(),
                deposit_amount: Uint128::zero(),
                link: None,
                execute_data: None,
                total_balance_at_end_poll: None,
                staked_amount: None,
            },
        )
        .unwrap();

    poll_store(&mut deps.storage)
        .save(
            &2u64.to_be_bytes(),
            &Poll {
                id: 1u64,
                creator: CanonicalAddr::from(vec![]),
                status: PollStatus::Passed,
                yes_votes: Uint128::zero(),
                no_votes: Uint128::zero(),
                end_height: 0u64,
                title: "title".to_string(),
                description: "description".to_string(),
                deposit_amount: Uint128::zero(),
                link: None,
                execute_data: None,
                total_balance_at_end_poll: None,
                staked_amount: None,
            },
        )
        .unwrap();

    let voter_addr_raw = deps.api.addr_canonicalize(TEST_VOTER).unwrap();
    poll_voter_store(&mut deps.storage, 1u64)
        .save(
            &voter_addr_raw.as_slice(),
            &VoterInfo {
                vote: VoteOption::Yes,
                balance: Uint128::from(5u128),
            },
        )
        .unwrap();
    poll_voter_store(&mut deps.storage, 2u64)
        .save(
            &voter_addr_raw.as_slice(),
            &VoterInfo {
                vote: VoteOption::Yes,
                balance: Uint128::from(5u128),
            },
        )
        .unwrap();
    bank_store(&mut deps.storage)
        .save(
            &voter_addr_raw.as_slice(),
            &TokenManager {
                share: Uint128::from(11u128),
                locked_balance: vec![
                    (
                        1u64,
                        VoterInfo {
                            vote: VoteOption::Yes,
                            balance: Uint128::from(5u128),
                        },
                    ),
                    (
                        2u64,
                        VoterInfo {
                            vote: VoteOption::Yes,
                            balance: Uint128::from(5u128),
                        },
                    ),
                ],
            },
        )
        .unwrap();

    // withdraw voting token must remove not in-progress votes infos from the store
    let info = mock_info(TEST_VOTER, &[]);
    let msg = ExecuteMsg::WithdrawVotingTokens {
        amount: Some(Uint128::from(5u128)),
    };

    let _ = execute(deps.as_mut(), mock_env(), info, msg).unwrap();
    let voter = poll_voter_read(&deps.storage, 1u64)
        .load(&voter_addr_raw.as_slice())
        .unwrap();
    assert_eq!(
        voter,
        VoterInfo {
            vote: VoteOption::Yes,
            balance: Uint128::from(5u128),
        }
    );
    assert!(poll_voter_read(&deps.storage, 2u64)
        .load(&voter_addr_raw.as_slice())
        .is_err(),);

    let token_manager = bank_read(&deps.storage)
        .load(&voter_addr_raw.as_slice())
        .unwrap();
    assert_eq!(
        token_manager.locked_balance,
        vec![(
            1u64,
            VoterInfo {
                vote: VoteOption::Yes,
                balance: Uint128::from(5u128),
            }
        )]
    );
}

#[test]
fn fails_withdraw_voting_tokens_no_stake() {
    let mut deps = mock_dependencies(&[]);
    instantiate::mock_instantiate(deps.as_mut());
    poll::mock_register_voting_token(deps.as_mut());

    let info = mock_info(TEST_VOTER, &coins(11, VOTING_TOKEN));
    let msg = ExecuteMsg::WithdrawVotingTokens {
        amount: Some(Uint128::from(11u128)),
    };

    let res = execute(deps.as_mut(), mock_env(), info, msg);

    match res {
        Ok(_) => panic!("Must return error"),
        Err(ContractError::NothingStaked {}) => (),
        Err(e) => panic!("Unexpected error: {:?}", e),
    }
}

#[test]
fn fails_withdraw_too_many_tokens() {
    let mut deps = mock_dependencies(&[]);
    instantiate::mock_instantiate(deps.as_mut());
    poll::mock_register_voting_token(deps.as_mut());

    deps.querier.with_token_balances(&[(
        &VOTING_TOKEN.to_string(),
        &[(&MOCK_CONTRACT_ADDR.to_string(), &Uint128::from(10u128))],
    )]);

    let msg = ExecuteMsg::Receive(Cw20ReceiveMsg {
        sender: TEST_VOTER.to_string(),
        amount: Uint128::from(10u128),
        msg: to_binary(&Cw20HookMsg::StakeVotingTokens {}).unwrap(),
    });

    let info = mock_info(VOTING_TOKEN, &[]);
    let execute_res = execute(deps.as_mut(), mock_env(), info, msg).unwrap();
    poll::assert_stake_tokens_result(10, 0, 10, 0, execute_res, deps.as_ref());

    let info = mock_info(TEST_VOTER, &[]);
    let msg = ExecuteMsg::WithdrawVotingTokens {
        amount: Some(Uint128::from(11u128)),
    };

    let res = execute(deps.as_mut(), mock_env(), info, msg);

    match res {
        Ok(_) => panic!("Must return error"),
        Err(ContractError::InvalidWithdrawAmount {}) => (),
        Err(e) => panic!("Unexpected error: {:?}", e),
    }
}

#[test]
fn fails_cast_vote_twice() {
    let mut deps = mock_dependencies(&[]);
    instantiate::mock_instantiate(deps.as_mut());
    poll::mock_register_voting_token(deps.as_mut());

    let env = common::mock_env_height(0, 10000);
    let info = mock_info(VOTING_TOKEN, &coins(2, VOTING_TOKEN));

    let msg = poll::create_poll_msg("test".to_string(), "test".to_string(), None, None);
    let execute_res = execute(deps.as_mut(), env.clone(), info, msg).unwrap();

    poll::assert_create_poll_result(
        1,
        env.block.height + DEFAULT_VOTING_PERIOD,
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
    poll::assert_stake_tokens_result(
        11,
        DEFAULT_PROPOSAL_DEPOSIT,
        11,
        1,
        execute_res,
        deps.as_ref(),
    );

    let amount = 1u128;
    let msg = ExecuteMsg::CastVote {
        poll_id: 1,
        vote: VoteOption::Yes,
        amount: Uint128::from(amount),
    };
    let env = common::mock_env_height(0, 10000);
    let info = mock_info(TEST_VOTER, &[]);
    let execute_res = execute(deps.as_mut(), env.clone(), info.clone(), msg).unwrap();
    poll::assert_cast_vote_success(TEST_VOTER, amount, 1, VoteOption::Yes, execute_res);

    let msg = ExecuteMsg::CastVote {
        poll_id: 1,
        vote: VoteOption::Yes,
        amount: Uint128::from(amount),
    };
    let res = execute(deps.as_mut(), env, info, msg);

    match res {
        Ok(_) => panic!("Must return error"),
        Err(ContractError::AlreadyVoted {}) => (),
        Err(e) => panic!("Unexpected error: {:?}", e),
    }
}

#[test]
fn fails_cast_vote_without_poll() {
    let mut deps = mock_dependencies(&[]);
    instantiate::mock_instantiate(deps.as_mut());
    poll::mock_register_voting_token(deps.as_mut());

    let msg = ExecuteMsg::CastVote {
        poll_id: 0,
        vote: VoteOption::Yes,
        amount: Uint128::from(1u128),
    };
    let info = mock_info(TEST_VOTER, &coins(11, VOTING_TOKEN));

    let res = execute(deps.as_mut(), mock_env(), info, msg);

    match res {
        Ok(_) => panic!("Must return error"),
        Err(ContractError::PollNotFound {}) => (),
        Err(e) => panic!("Unexpected error: {:?}", e),
    }
}

#[test]
fn successful_stake_voting_tokens() {
    let mut deps = mock_dependencies(&[]);
    instantiate::mock_instantiate(deps.as_mut());
    poll::mock_register_voting_token(deps.as_mut());

    deps.querier.with_token_balances(&[(
        &VOTING_TOKEN.to_string(),
        &[(&MOCK_CONTRACT_ADDR.to_string(), &Uint128::from(11u128))],
    )]);

    let msg = ExecuteMsg::Receive(Cw20ReceiveMsg {
        sender: TEST_VOTER.to_string(),
        amount: Uint128::from(11u128),
        msg: to_binary(&Cw20HookMsg::StakeVotingTokens {}).unwrap(),
    });

    let info = mock_info(VOTING_TOKEN, &[]);
    let execute_res = execute(deps.as_mut(), mock_env(), info, msg).unwrap();
    poll::assert_stake_tokens_result(11, 0, 11, 0, execute_res, deps.as_ref());
}

#[test]
fn fails_insufficient_funds() {
    let mut deps = mock_dependencies(&[]);

    // initialize the store
    instantiate::mock_instantiate(deps.as_mut());
    poll::mock_register_voting_token(deps.as_mut());

    // insufficient token
    let msg = ExecuteMsg::Receive(Cw20ReceiveMsg {
        sender: TEST_VOTER.to_string(),
        amount: Uint128::from(0u128),
        msg: to_binary(&Cw20HookMsg::StakeVotingTokens {}).unwrap(),
    });

    let info = mock_info(VOTING_TOKEN, &[]);
    let res = execute(deps.as_mut(), mock_env(), info, msg);

    match res {
        Ok(_) => panic!("Must return error"),
        Err(ContractError::InsufficientFunds {}) => (),
        Err(e) => panic!("Unexpected error: {:?}", e),
    }
}

#[test]
fn fails_staking_wrong_token() {
    let mut deps = mock_dependencies(&[]);

    // initialize the store
    instantiate::mock_instantiate(deps.as_mut());
    poll::mock_register_voting_token(deps.as_mut());

    deps.querier.with_token_balances(&[(
        &VOTING_TOKEN.to_string(),
        &[(&MOCK_CONTRACT_ADDR.to_string(), &Uint128::from(11u128))],
    )]);

    // wrong token
    let msg = ExecuteMsg::Receive(Cw20ReceiveMsg {
        sender: TEST_VOTER.to_string(),
        amount: Uint128::from(11u128),
        msg: to_binary(&Cw20HookMsg::StakeVotingTokens {}).unwrap(),
    });

    let info = mock_info(&(VOTING_TOKEN.to_string() + "2"), &[]);
    let res = execute(deps.as_mut(), mock_env(), info, msg);

    match res {
        Ok(_) => panic!("Must return error"),
        Err(ContractError::Unauthorized {}) => (),
        Err(e) => panic!("Unexpected error: {:?}", e),
    }
}
