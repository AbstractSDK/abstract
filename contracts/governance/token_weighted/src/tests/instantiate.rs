use cosmwasm_std::testing::{mock_env, mock_info, MOCK_CONTRACT_ADDR};
use cosmwasm_std::{coins, from_binary, DepsMut};
use cosmwasm_std::{Api, CanonicalAddr, Decimal, Uint128};

use crate::contract::{execute, instantiate, query};
use crate::{ExecuteMsg, InstantiateMsg, QueryMsg};
use crate::state::{config_read, state_read, Config, ConfigResponse, State};
use crate::tests::common::{
    DEFAULT_EXPIRATION_PERIOD, DEFAULT_FIX_PERIOD, DEFAULT_PROPOSAL_DEPOSIT, DEFAULT_QUORUM,
    DEFAULT_THRESHOLD, DEFAULT_TIMELOCK_PERIOD, DEFAULT_VOTING_PERIOD, TEST_CREATOR, VOTING_TOKEN,
};
use crate::tests::mock_querier::mock_dependencies;
use crate::tests::poll::mock_register_voting_token;
use crate::ContractError;

pub(crate) fn instantiate_msg() -> InstantiateMsg {
    InstantiateMsg {
        quorum: Decimal::percent(DEFAULT_QUORUM),
        threshold: Decimal::percent(DEFAULT_THRESHOLD),
        voting_period: DEFAULT_VOTING_PERIOD,
        timelock_period: DEFAULT_TIMELOCK_PERIOD,
        expiration_period: DEFAULT_EXPIRATION_PERIOD,
        proposal_deposit: Uint128::from(DEFAULT_PROPOSAL_DEPOSIT),
        snapshot_period: DEFAULT_FIX_PERIOD,
    }
}

/**
 * Mocks instantiation.
 */
pub fn mock_instantiate(deps: DepsMut) {
    let msg = InstantiateMsg {
        quorum: Decimal::percent(DEFAULT_QUORUM),
        threshold: Decimal::percent(DEFAULT_THRESHOLD),
        voting_period: DEFAULT_VOTING_PERIOD,
        timelock_period: DEFAULT_TIMELOCK_PERIOD,
        expiration_period: DEFAULT_EXPIRATION_PERIOD,
        proposal_deposit: Uint128::from(DEFAULT_PROPOSAL_DEPOSIT),
        snapshot_period: DEFAULT_FIX_PERIOD,
    };

    let info = mock_info(TEST_CREATOR, &[]);
    let _res = instantiate(deps, mock_env(), info, msg)
        .expect("contract successfully handles InstantiateMsg");
}

/**
 * Tests successful instantiation of the contract.
 */
#[test]
fn successful_initialization() {
    let mut deps = mock_dependencies(&[]);

    let msg = instantiate_msg();
    let info = mock_info(TEST_CREATOR, &coins(2, VOTING_TOKEN));
    let res = instantiate(deps.as_mut(), mock_env(), info.clone(), msg).unwrap();
    assert_eq!(0, res.messages.len());

    let config: Config = config_read(deps.as_ref().storage).load().unwrap();
    assert_eq!(
        config,
        Config {
            whale_token: CanonicalAddr::from(vec![]),
            owner: deps.api.addr_canonicalize(&TEST_CREATOR).unwrap(),
            quorum: Decimal::percent(DEFAULT_QUORUM),
            threshold: Decimal::percent(DEFAULT_THRESHOLD),
            voting_period: DEFAULT_VOTING_PERIOD,
            timelock_period: DEFAULT_TIMELOCK_PERIOD,
            expiration_period: DEFAULT_EXPIRATION_PERIOD,
            proposal_deposit: Uint128::from(DEFAULT_PROPOSAL_DEPOSIT),
            snapshot_period: DEFAULT_FIX_PERIOD
        }
    );

    let msg = ExecuteMsg::RegisterContracts {
        whale_token: VOTING_TOKEN.to_string(),
    };
    let _res = execute(deps.as_mut(), mock_env(), info, msg).unwrap();
    let config: Config = config_read(deps.as_ref().storage).load().unwrap();
    assert_eq!(
        config.whale_token,
        deps.api.addr_canonicalize(&VOTING_TOKEN).unwrap()
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

/**
 * Tests unsuccessful instantiation of the contract.
 */
#[test]
#[should_panic]
fn invalid_quorum_fails_initialization() {
    let mut deps = mock_dependencies(&[]);

    let mut msg = instantiate_msg();
    msg.quorum = Decimal::from_ratio(2u128, 1u128);

    let info = mock_info(TEST_CREATOR, &coins(2, VOTING_TOKEN));
    instantiate(deps.as_mut(), mock_env(), info.clone(), msg).unwrap();
}

#[test]
#[should_panic]
fn invalid_threshold_fails_initialization() {
    let mut deps = mock_dependencies(&[]);

    let mut msg = instantiate_msg();
    msg.threshold = Decimal::from_ratio(2u128, 1u128);

    let info = mock_info(TEST_CREATOR, &coins(2, VOTING_TOKEN));
    instantiate(deps.as_mut(), mock_env(), info.clone(), msg).unwrap();
}

/**
 * Tests updating the configuration of the contract.
 */
#[test]
fn successful_update_config() {
    let mut deps = mock_dependencies(&[]);
    mock_instantiate(deps.as_mut());
    mock_register_voting_token(deps.as_mut());

    // update owner
    let info = mock_info(TEST_CREATOR, &[]);
    let msg = ExecuteMsg::UpdateConfig {
        owner: Some("addr0001".to_string()),
        quorum: None,
        threshold: None,
        voting_period: None,
        timelock_period: None,
        expiration_period: None,
        proposal_deposit: None,
        snapshot_period: None,
    };

    let res = execute(deps.as_mut(), mock_env(), info, msg).unwrap();
    assert_eq!(0, res.messages.len());

    // it worked, let's query the state
    let res = query(deps.as_ref(), mock_env(), QueryMsg::Config {}).unwrap();
    let config: ConfigResponse = from_binary(&res).unwrap();
    assert_eq!("addr0001", config.owner.as_str());
    assert_eq!(Decimal::percent(DEFAULT_QUORUM), config.quorum);
    assert_eq!(Decimal::percent(DEFAULT_THRESHOLD), config.threshold);
    assert_eq!(DEFAULT_VOTING_PERIOD, config.voting_period);
    assert_eq!(DEFAULT_TIMELOCK_PERIOD, config.timelock_period);
    assert_eq!(DEFAULT_PROPOSAL_DEPOSIT, config.proposal_deposit.u128());

    // update left items with the new owner
    let info = mock_info("addr0001", &[]);
    let msg = ExecuteMsg::UpdateConfig {
        owner: None,
        quorum: Some(Decimal::percent(20)),
        threshold: Some(Decimal::percent(75)),
        voting_period: Some(20000u64),
        timelock_period: Some(20000u64),
        expiration_period: Some(30000u64),
        proposal_deposit: Some(Uint128::from(123u128)),
        snapshot_period: Some(11),
    };

    let res = execute(deps.as_mut(), mock_env(), info, msg).unwrap();
    assert_eq!(0, res.messages.len());

    // it worked, let's query the state
    let res = query(deps.as_ref(), mock_env(), QueryMsg::Config {}).unwrap();
    let config: ConfigResponse = from_binary(&res).unwrap();
    assert_eq!("addr0001", config.owner.as_str());
    assert_eq!(Decimal::percent(20), config.quorum);
    assert_eq!(Decimal::percent(75), config.threshold);
    assert_eq!(20000u64, config.voting_period);
    assert_eq!(20000u64, config.timelock_period);
    assert_eq!(30000u64, config.expiration_period);
    assert_eq!(123u128, config.proposal_deposit.u128());
    assert_eq!(11u64, config.snapshot_period);
}

#[test]
fn unsuccessful_update_config() {
    let mut deps = mock_dependencies(&[]);
    mock_instantiate(deps.as_mut());
    mock_register_voting_token(deps.as_mut());

    // Unauthorized user
    let info = mock_info("unauthorized_addr", &[]);
    let msg = ExecuteMsg::UpdateConfig {
        owner: None,
        quorum: None,
        threshold: None,
        voting_period: None,
        timelock_period: None,
        expiration_period: None,
        proposal_deposit: None,
        snapshot_period: None,
    };

    let res = execute(deps.as_mut(), mock_env(), info, msg);
    match res {
        Err(ContractError::Unauthorized {}) => (),
        _ => panic!("Must return unauthorized error"),
    }
}
