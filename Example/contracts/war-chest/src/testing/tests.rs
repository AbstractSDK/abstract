use crate::contract::{execute, instantiate, query};
use crate::error::ContractError;
use crate::msg::{ConfigResponse, ExecuteMsg, InstantiateMsg, QueryMsg};
use cosmwasm_std::testing::{mock_dependencies, mock_env, mock_info};
use cosmwasm_std::{from_binary, to_binary, CosmosMsg, SubMsg, Uint128, WasmMsg};
use cw20::Cw20ExecuteMsg;
use cw_controllers::AdminError;

fn init_msg() -> InstantiateMsg {
    InstantiateMsg {
        admin_addr: "gov".to_string(),
        whale_token_addr: "whale".to_string(),
        spend_limit: Uint128::from(1_000_000u128),
    }
}

#[test]
fn proper_initialization() {
    let mut deps = mock_dependencies(&[]);
    let msg = init_msg();
    let info = mock_info("addr0000", &[]);

    let _res = instantiate(deps.as_mut(), mock_env(), info, msg).unwrap();

    let config: ConfigResponse =
        from_binary(&query(deps.as_ref(), mock_env(), QueryMsg::GetConfig {}).unwrap()).unwrap();
    assert_eq!("whale", config.whale_token_addr.as_str());
    assert_eq!(Uint128::from(1_000_000u128), config.spend_limit);
}

#[test]
fn test_update_spend_limit() {
    let mut deps = mock_dependencies(&[]);
    let msg = init_msg();
    let info = mock_info("addr0000", &[]);

    let _res = instantiate(deps.as_mut(), mock_env(), info, msg).unwrap();

    let msg = ExecuteMsg::UpdateSpendLimit {
        spend_limit: Uint128::from(500000u128),
    };
    let info = mock_info("addr0000", &[]);
    match execute(deps.as_mut(), mock_env(), info, msg.clone()) {
        Ok(_) => panic!("Must return error"),
        Err(ContractError::Admin(AdminError::NotAdmin {})) => (),
        Err(_) => panic!("Unknown error"),
    }

    let info = mock_info("gov", &[]);
    let _res = execute(deps.as_mut(), mock_env(), info, msg).unwrap();
    let config: ConfigResponse =
        from_binary(&query(deps.as_ref(), mock_env(), QueryMsg::GetConfig {}).unwrap()).unwrap();
    assert_eq!(config.spend_limit, Uint128::from(500000u128));
}

#[test]
fn test_spend_fails_if_not_admin() {
    let mut deps = mock_dependencies(&[]);
    let msg = init_msg();
    let info = mock_info("addr0000", &[]);

    let _res = instantiate(deps.as_mut(), mock_env(), info, msg).unwrap();

    let msg = ExecuteMsg::Spend {
        recipient: "addr0000".to_string(),
        amount: Uint128::from(1000000u128),
    };

    let info = mock_info("addr0000", &[]);

    match execute(deps.as_mut(), mock_env(), info, msg) {
        Ok(_) => panic!("Must return error"),
        Err(ContractError::Admin(AdminError::NotAdmin {})) => (),
        Err(_) => panic!("Unknown error"),
    }
}

#[test]
fn test_spend_fails_if_spend_limit_is_exceeded() {
    let mut deps = mock_dependencies(&[]);
    let msg = init_msg();
    let info = mock_info("addr0000", &[]);

    let _res = instantiate(deps.as_mut(), mock_env(), info, msg).unwrap();

    let msg = ExecuteMsg::Spend {
        recipient: "addr0000".to_string(),
        amount: Uint128::from(2000000u128),
    };

    let info = mock_info("gov", &[]);
    match execute(deps.as_mut(), mock_env(), info, msg) {
        Ok(_) => panic!("Must return error"),
        Err(ContractError::TooMuchSpend { .. }) => (),
        Err(_) => panic!("Unknown error"),
    }
}

#[test]
fn test_spend() {
    let mut deps = mock_dependencies(&[]);
    let msg = init_msg();
    let info = mock_info("addr0000", &[]);

    let _res = instantiate(deps.as_mut(), mock_env(), info, msg).unwrap();

    let msg = ExecuteMsg::Spend {
        recipient: "addr0000".to_string(),
        amount: Uint128::from(1000000u128),
    };

    let info = mock_info("gov", &[]);
    let res = execute(deps.as_mut(), mock_env(), info, msg).unwrap();
    assert_eq!(
        res.messages,
        vec![SubMsg::new(CosmosMsg::Wasm(WasmMsg::Execute {
            contract_addr: "whale".to_string(),
            funds: vec![],
            msg: to_binary(&Cw20ExecuteMsg::Transfer {
                recipient: "addr0000".to_string(),
                amount: Uint128::from(1000000u128),
            })
            .unwrap(),
        }))]
    );
}
