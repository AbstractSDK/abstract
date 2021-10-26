use cosmwasm_std::testing::{mock_env, mock_info};
use cosmwasm_std::Api;
use cosmwasm_std::{coins, from_binary, to_binary, DepsMut, MessageInfo, ReplyOn, SubMsg, WasmMsg};

use crate::contract::{execute, instantiate};
use crate::state::{State, ARB_BASE_ASSET, STATE};

use terraswap::asset::AssetInfo;

use white_whale::deposit_info::ArbBaseAsset;

use crate::tests::common::{TEST_CREATOR, VAULT_CONTRACT};

use crate::msg::*;
use crate::tests::mock_querier::mock_dependencies;

pub(crate) fn instantiate_msg() -> InitMsg {
    InitMsg {
        trader: "test_trader".to_string(),
        vault_address: VAULT_CONTRACT.to_string(),
        seignorage_address: "seignorage".to_string(),
        pool_address: "terraswap_pool".to_string(),
        asset_info: AssetInfo::NativeToken {
            denom: "uusd".to_string(),
        },
    }
}

/**
 * Mocks instantiation.
 */
pub fn mock_instantiate(deps: DepsMut) {
    let msg = InitMsg {
        trader: "test_trader".to_string(),
        vault_address: VAULT_CONTRACT.to_string(),
        seignorage_address: "seignorage".to_string(),
        pool_address: "terraswap_pool".to_string(),
        asset_info: AssetInfo::NativeToken {
            denom: "uusd".to_string(),
        },
    };

    let info = mock_info(TEST_CREATOR, &[]);
    let _res =
        instantiate(deps, mock_env(), info, msg).expect("contract successfully handles InitMsg");
}

/**
 * Tests successful instantiation of the contract.
 */
#[test]
fn successful_initialization() {
    let mut deps = mock_dependencies(&[]);

    let msg = instantiate_msg();
    let info = mock_info(TEST_CREATOR, &[]);
    let res = instantiate(deps.as_mut(), mock_env(), info.clone(), msg).unwrap();
    assert_eq!(0, res.messages.len());

    let state: State = STATE.load(&deps.storage).unwrap();
    assert_eq!(
        state,
        State {
            trader: deps.api.addr_canonicalize(&"test_trader").unwrap(),
            vault_address: deps.api.addr_canonicalize(&VAULT_CONTRACT).unwrap(),
            seignorage_address: deps.api.addr_canonicalize(&"seignorage").unwrap(),
            pool_address: deps.api.addr_canonicalize(&"terraswap_pool ").unwrap(),
        }
    );

    let base_asset: ArbBaseAsset = ARB_BASE_ASSET.load(&deps.storage).unwrap();
    assert_eq!(
        base_asset.asset_info,
        AssetInfo::NativeToken {
            denom: "uusd".to_string(),
        },
    );
}

#[test]
fn successfull_set_admin() {
    let mut deps = mock_dependencies(&[]);
    mock_instantiate(deps.as_mut());

    // update admin
    let info = mock_info(TEST_CREATOR, &[]);
    let msg = ExecuteMsg::SetAdmin {
        admin: "new_admin".to_string(),
    };

    let res = execute(deps.as_mut(), mock_env(), info, msg).unwrap();
    assert_eq!(0, res.messages.len());
}
