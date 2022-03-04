use std::panic;

use crate::contract::{execute, instantiate};
use crate::error::*;
use crate::tests::common::TEST_CREATOR;
use cosmwasm_std::testing::{mock_dependencies, mock_env, mock_info};
use cosmwasm_std::{to_binary, Addr, QuerierWrapper, ReplyOn, SubMsg, Uint128, WasmMsg};
use cw20::Cw20ExecuteMsg;
use pandora_os::core::treasury::msg::{ExecuteMsg, InstantiateMsg};
use terraswap::asset::{Asset, AssetInfo};

const NOT_ALLOWED: &str = "some_other_contract";

fn init_msg() -> InstantiateMsg {
    InstantiateMsg {}
}

#[test]
fn test_non_whitelisted() {
    let mut deps = mock_dependencies(&[]);
    let msg = init_msg();
    let info = mock_info(TEST_CREATOR, &[]);
    let _res = instantiate(deps.as_mut(), mock_env(), info.clone(), msg).unwrap();

    let msg = ExecuteMsg::AddDApp {
        dapp: TEST_CREATOR.to_string(),
    };

    match execute(deps.as_mut(), mock_env(), info.clone(), msg) {
        Ok(_) => (),
        Err(_) => panic!("Unknown error"),
    }

    let test_token = Asset {
        info: AssetInfo::Token {
            contract_addr: "test_token".to_string(),
        },
        amount: Uint128::zero(),
    };

    let info = mock_info(NOT_ALLOWED, &[]);

    let msg = ExecuteMsg::DAppAction {
        msgs: vec![test_token
            .into_msg(
                &QuerierWrapper::new(&deps.querier),
                Addr::unchecked(NOT_ALLOWED),
            )
            .unwrap()],
    };

    match execute(deps.as_mut(), mock_env(), info.clone(), msg) {
        Ok(_) => panic!("Sender should not be allowed to do this action"),
        Err(e) => match e {
            TreasuryError::SenderNotWhitelisted {} => (),
            _ => panic!("Unknown error: {}", e),
        },
    }
}

#[test]
fn test_whitelisted() {
    let mut deps = mock_dependencies(&[]);
    let msg = init_msg();
    let info = mock_info(TEST_CREATOR, &[]);
    let _res = instantiate(deps.as_mut(), mock_env(), info.clone(), msg).unwrap();

    let msg = ExecuteMsg::AddDApp {
        dapp: TEST_CREATOR.to_string(),
    };

    match execute(deps.as_mut(), mock_env(), info.clone(), msg) {
        Ok(_) => (),
        Err(_) => panic!("Unknown error"),
    }

    let test_token = Asset {
        info: AssetInfo::Token {
            contract_addr: "test_token".to_string(),
        },
        amount: Uint128::from(10_000u64),
    };

    let info = mock_info(TEST_CREATOR, &[]);

    let msg = ExecuteMsg::DAppAction {
        msgs: vec![test_token
            .into_msg(
                &QuerierWrapper::new(&deps.querier),
                Addr::unchecked(TEST_CREATOR),
            )
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
