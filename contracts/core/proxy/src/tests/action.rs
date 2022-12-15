use std::panic;

use crate::contract::{execute, instantiate};
use crate::error::*;
use crate::tests::common::TEST_CREATOR;
use abstract_sdk::os::proxy::{ExecuteMsg, InstantiateMsg};
use cosmwasm_std::testing::{mock_dependencies, mock_env, mock_info};
use cosmwasm_std::{to_binary, Addr, ReplyOn, SubMsg, Uint128, WasmMsg};
use cw20::Cw20ExecuteMsg;
use cw_asset::{Asset, AssetInfo};

const NOT_ALLOWED: &str = "some_other_contract";

fn init_msg(os_id: u32) -> InstantiateMsg {
    InstantiateMsg {
        os_id,
        ans_host_address: "".into(),
    }
}

#[test]
fn test_non_whitelisted() {
    let mut deps = mock_dependencies();
    let msg = init_msg(0);
    let info = mock_info(TEST_CREATOR, &[]);
    let _res = instantiate(deps.as_mut(), mock_env(), info.clone(), msg).unwrap();

    let msg = ExecuteMsg::AddModule {
        module: TEST_CREATOR.to_string(),
    };

    match execute(deps.as_mut(), mock_env(), info.clone(), msg) {
        Ok(_) => (),
        Err(_) => panic!("Unknown error"),
    }

    let test_token = Asset {
        info: AssetInfo::Cw20(Addr::unchecked("test_token".to_string())),
        amount: Uint128::zero(),
    };

    let info = mock_info(NOT_ALLOWED, &[]);

    let msg = ExecuteMsg::ModuleAction {
        msgs: vec![test_token
            .transfer_msg(Addr::unchecked(NOT_ALLOWED))
            .unwrap()],
    };

    match execute(deps.as_mut(), mock_env(), info.clone(), msg) {
        Ok(_) => panic!("Sender should not be allowed to do this action"),
        Err(e) => match e {
            ProxyError::SenderNotWhitelisted {} => (),
            _ => panic!("Unknown error: {}", e),
        },
    }
}

#[test]
fn test_whitelisted() {
    let mut deps = mock_dependencies();
    let msg = init_msg(0);
    let info = mock_info(TEST_CREATOR, &[]);
    let _res = instantiate(deps.as_mut(), mock_env(), info.clone(), msg).unwrap();

    let msg = ExecuteMsg::AddModule {
        module: TEST_CREATOR.to_string(),
    };

    match execute(deps.as_mut(), mock_env(), info.clone(), msg) {
        Ok(_) => (),
        Err(_) => panic!("Unknown error"),
    }

    let test_token = Asset {
        info: AssetInfo::Cw20(Addr::unchecked("test_token".to_string())),
        amount: Uint128::from(10_000u64),
    };

    let info = mock_info(TEST_CREATOR, &[]);

    let msg = ExecuteMsg::ModuleAction {
        msgs: vec![test_token
            .transfer_msg(Addr::unchecked(TEST_CREATOR))
            .unwrap()],
    };

    match execute(deps.as_mut(), mock_env(), info.clone(), msg) {
        Ok(res) => {
            assert_eq!(
                res.messages,
                vec![SubMsg {
                    // Create LP token
                    msg: WasmMsg::Execute {
                        contract_addr: "test_token".to_string(),
                        msg: to_binary(&Cw20ExecuteMsg::Transfer {
                            recipient: TEST_CREATOR.to_string(),
                            amount: Uint128::from(10_000u64)
                        })
                        .unwrap(),
                        funds: vec![],
                    }
                    .into(),
                    gas_limit: None,
                    id: 0u64,
                    reply_on: ReplyOn::Never,
                }]
            );
        }
        Err(e) => panic!("Unknown error: {}", e),
    }
}
