use cosmwasm_std::testing::{mock_env, mock_info, MOCK_CONTRACT_ADDR};
use cosmwasm_std::{attr, from_binary, to_binary, Addr, Uint128};
use cw20::Cw20ReceiveMsg;

use crate::contract::{execute, query};
use crate::{ExecuteMsg, QueryMsg};
use crate::staking::stake_voting_tokens;
use crate::state::{Cw20HookMsg, StakerResponse};
use crate::tests::common::{TEST_VOTER, VOTING_TOKEN};
use crate::tests::mock_querier::mock_dependencies;
use crate::tests::{instantiate, poll};
use crate::ContractError;

#[test]
fn share_calculation() {
    let mut deps = mock_dependencies(&[]);

    // initialize the store
    instantiate::mock_instantiate(deps.as_mut());
    poll::mock_register_voting_token(deps.as_mut());

    // create 100 share
    deps.querier.with_token_balances(&[(
        &VOTING_TOKEN.to_string(),
        &[(&MOCK_CONTRACT_ADDR.to_string(), &Uint128::from(100u128))],
    )]);

    let msg = ExecuteMsg::Receive(Cw20ReceiveMsg {
        sender: TEST_VOTER.to_string(),
        amount: Uint128::from(100u128),
        msg: to_binary(&Cw20HookMsg::StakeVotingTokens {}).unwrap(),
    });

    let info = mock_info(VOTING_TOKEN, &[]);
    let _res = execute(deps.as_mut(), mock_env(), info, msg);

    // add more balance(100) to make share:balance = 1:2
    deps.querier.with_token_balances(&[(
        &VOTING_TOKEN.to_string(),
        &[(
            &MOCK_CONTRACT_ADDR.to_string(),
            &Uint128::from(200u128 + 100u128),
        )],
    )]);

    let msg = ExecuteMsg::Receive(Cw20ReceiveMsg {
        sender: TEST_VOTER.to_string(),
        amount: Uint128::from(100u128),
        msg: to_binary(&Cw20HookMsg::StakeVotingTokens {}).unwrap(),
    });

    let info = mock_info(VOTING_TOKEN, &[]);
    let res = execute(deps.as_mut(), mock_env(), info, msg).unwrap();
    assert_eq!(
        res.attributes,
        vec![
            attr("action", "staking"),
            attr("sender", TEST_VOTER),
            attr("share", "50"),
            attr("amount", "100"),
        ]
    );

    let msg = ExecuteMsg::WithdrawVotingTokens {
        amount: Some(Uint128::from(100u128)),
    };
    let info = mock_info(TEST_VOTER, &[]);
    let res = execute(deps.as_mut(), mock_env(), info, msg).unwrap();
    assert_eq!(
        res.attributes,
        vec![
            attr("action", "withdraw"),
            attr("recipient", TEST_VOTER),
            attr("amount", "100"),
        ]
    );

    // 100 tokens withdrawn
    deps.querier.with_token_balances(&[(
        &VOTING_TOKEN.to_string(),
        &[(&MOCK_CONTRACT_ADDR.to_string(), &Uint128::from(200u128))],
    )]);

    let res = query(
        deps.as_ref(),
        mock_env(),
        QueryMsg::Staker {
            address: TEST_VOTER.to_string(),
        },
    )
    .unwrap();
    let stake_info: StakerResponse = from_binary(&res).unwrap();
    assert_eq!(stake_info.share, Uint128::new(100));
    assert_eq!(stake_info.balance, Uint128::new(200));
    assert_eq!(stake_info.locked_balance, vec![]);
}

#[test]
fn fails_insufficient_funds_staking() {
    let mut deps = mock_dependencies(&[]);

    match stake_voting_tokens(deps.as_mut(), Addr::unchecked(""), Uint128::zero()) {
        Ok(_) => panic!("Must return error"),
        Err(ContractError::InsufficientFunds {}) => (),
        Err(_) => panic!("Unknown error"),
    }
}
