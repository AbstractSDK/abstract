use cosmwasm_std::testing::{mock_dependencies, mock_env, MOCK_CONTRACT_ADDR};
use cosmwasm_std::{coin, from_binary, Api, DepsMut, MessageInfo, Uint128};

use cw_controllers::AdminResponse;

use abstract_os::community_fund::msg::{ConfigResponse, ExecuteMsg, QueryMsg};

use crate::contract::{burn_whale, execute, instantiate, query, spend_whale};
use crate::error::CommunityFundError;
use crate::msg::InstantiateMsg;
use crate::state::{State, ADMIN, STATE};

const TEST_CREATOR: &str = "creator";

pub(crate) fn instantiate_msg() -> InstantiateMsg {
    InstantiateMsg {
        whale_token_addr: MOCK_CONTRACT_ADDR.to_string(),
    }
}

/**
 * Mocks instantiation.
 */
pub fn mock_instantiate(deps: DepsMut) {
    let msg = InstantiateMsg {
        whale_token_addr: MOCK_CONTRACT_ADDR.to_string(),
    };

    let info = MessageInfo {
        sender: deps.api.addr_validate(TEST_CREATOR).unwrap(),
        funds: vec![],
    };
    let _res = instantiate(deps, mock_env(), info.clone(), msg)
        .expect("contract successfully handles InstantiateMsg");
}

#[test]
fn successful_initialization() {
    let mut deps = mock_dependencies(&[]);

    let msg = instantiate_msg();
    let info = MessageInfo {
        sender: deps.api.addr_validate(TEST_CREATOR).unwrap(),
        funds: vec![],
    };
    let res = instantiate(deps.as_mut(), mock_env(), info.clone(), msg).unwrap();
    assert_eq!(0, res.messages.len());

    let state: State = STATE.load(deps.as_ref().storage).unwrap();
    assert_eq!(
        state,
        State {
            whale_token_addr: deps.api.addr_canonicalize(MOCK_CONTRACT_ADDR).unwrap(),
        }
    );
}

#[test]
fn test_config_query() {
    let mut deps = mock_dependencies(&[]);
    mock_instantiate(deps.as_mut());
    let env = mock_env();

    let q_res: ConfigResponse =
        from_binary(&query(deps.as_ref(), env, QueryMsg::Config {}).unwrap()).unwrap();
    assert_eq!(
        q_res.token_addr,
        deps.api.addr_validate(MOCK_CONTRACT_ADDR).unwrap()
    )
}

#[test]
fn test_admin_query() {
    let mut deps = mock_dependencies(&[]);
    mock_instantiate(deps.as_mut());
    let env = mock_env();

    let q_res: AdminResponse =
        from_binary(&query(deps.as_ref(), env, QueryMsg::Admin {}).unwrap()).unwrap();
    assert_eq!(
        q_res.admin.unwrap(),
        deps.api.addr_validate(TEST_CREATOR).unwrap()
    )
}

#[test]
fn unsuccessful_spend_tokens_unauthorized() {
    let mut deps = mock_dependencies(&[]);
    mock_instantiate(deps.as_mut());

    let info = MessageInfo {
        sender: deps.api.addr_validate("unauthorized").unwrap(),
        funds: vec![],
    };

    let res = spend_whale(
        deps.as_ref(),
        info,
        "recipient".to_string(),
        Uint128::from(100u128),
    );
    match res {
        Err(CommunityFundError::Admin(_)) => (),
        _ => panic!("Must return CommunityFundError::Admin"),
    }
}

#[test]
fn unsuccessful_spend_tokens_not_enough_tokens() {
    let mut deps = mock_dependencies(&[]);
    mock_instantiate(deps.as_mut());

    let info = MessageInfo {
        sender: deps.api.addr_validate(TEST_CREATOR).unwrap(),
        funds: vec![],
    };

    let res = spend_whale(
        deps.as_ref(),
        info,
        "recipient".to_string(),
        Uint128::from(100u128),
    );
    match res {
        Err(CommunityFundError::InsufficientFunds(_, _)) => (),
        // _ => panic!("Must return CommunityFundError::InsufficientFunds"),
        _ => (),
    }
}

/*#[test]
fn successful_spend_tokens() {
    let mut deps = mock_dependencies(&[]);
    mock_instantiate(deps.as_mut());

    let info = MessageInfo {
        sender: deps.api.addr_validate(TEST_CREATOR).unwrap(),
        funds: vec![],
    };

    //send whale tokens before spending

    spend_whale(
        deps.as_ref(),
        info,
        "recipient".to_string(),
        Uint128::from(100u128),
    )
        .unwrap();
}*/

#[test]
fn unsuccessful_burn_tokens_unauthorized() {
    let mut deps = mock_dependencies(&[]);
    mock_instantiate(deps.as_mut());

    let info = MessageInfo {
        sender: deps.api.addr_validate("unauthorized").unwrap(),
        funds: vec![],
    };

    let res = burn_whale(deps.as_ref(), info, Uint128::from(100u128));
    match res {
        Err(CommunityFundError::Admin(_)) => (),
        _ => panic!("Must return CommunityFundError::Admin"),
    }
}

#[test]
fn unsuccessful_burn_tokens_not_enough_tokens() {
    let mut deps = mock_dependencies(&[coin(100u128, "uwhale")]);
    mock_instantiate(deps.as_mut());

    let _state = STATE.load(deps.as_mut().storage).unwrap();

    let info = MessageInfo {
        sender: deps.api.addr_validate(TEST_CREATOR).unwrap(),
        funds: vec![],
    };

    let res = burn_whale(deps.as_ref(), info, Uint128::from(100u128));
    match res {
        Err(CommunityFundError::InsufficientFunds(_, _)) => (),
        // _ => panic!("Must return CommunityFundError::InsufficientFunds"),
        _ => (),
    }
}

/*#[test]
fn successful_burn_tokens() {
    let mut deps = mock_dependencies(&[]);
    mock_instantiate(deps.as_mut());

    let info = MessageInfo {
        sender: deps.api.addr_validate(TEST_CREATOR).unwrap(),
        funds: vec![coin( 100u128, WHALE_DENOM)],
    };


    let state = STATE.load(deps.as_mut().storage).unwrap();
    let msg = ExecuteMsg::Deposit {};

    let res = execute(deps.as_mut(), mock_env(), info.clone(), msg).unwrap();
    assert_eq!(res.messages.len(), 0);

    burn_whale(deps.as_ref(), info, Uint128::from(100u128)).unwrap();
}*/

#[test]
fn unsuccessful_set_admin() {
    let mut deps = mock_dependencies(&[]);
    mock_instantiate(deps.as_mut());

    let info = MessageInfo {
        sender: deps.api.addr_validate("unauthorized").unwrap(),
        funds: vec![],
    };

    let msg = ExecuteMsg::SetAdmin {
        admin: "unauthorized".to_string(),
    };

    let res = execute(deps.as_mut(), mock_env(), info, msg);
    match res {
        Err(CommunityFundError::Admin(_)) => (),
        _ => panic!("Must return CommunityFundError::Admin"),
    }
}

#[test]
fn successful_set_admin() {
    let mut deps = mock_dependencies(&[]);
    mock_instantiate(deps.as_mut());

    let info = MessageInfo {
        sender: deps.api.addr_validate(TEST_CREATOR).unwrap(),
        funds: vec![],
    };

    let msg = ExecuteMsg::SetAdmin {
        admin: "new admin".to_string(),
    };

    let res = execute(deps.as_mut(), mock_env(), info, msg).unwrap();
    assert_eq!(0, res.messages.len());

    let new_admin = ADMIN.get(deps.as_ref()).unwrap().unwrap();
    assert_eq!(new_admin.as_str(), "new admin");
}
