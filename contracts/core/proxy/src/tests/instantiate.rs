use cosmwasm_std::testing::{
    mock_dependencies, mock_env, mock_info, MockApi, MockQuerier, MockStorage, MOCK_CONTRACT_ADDR,
};
use cosmwasm_std::{Addr, Api, DepsMut};
use cosmwasm_std::{OwnedDeps, Uint128};

use crate::contract::{execute, instantiate, ProxyResult};

use abstract_os::objects::proxy_asset::{ProxyAsset, UncheckedProxyAsset};
use abstract_sdk::os::proxy::state::*;
use abstract_sdk::os::proxy::*;
use cw_asset::{Asset, AssetInfo};
use speculoos::prelude::*;

use crate::tests::common::{DAPP, TEST_CREATOR};

pub fn instantiate_msg(os_id: u32) -> InstantiateMsg {
    InstantiateMsg {
        os_id,
        ans_host_address: MOCK_CONTRACT_ADDR.into(),
    }
}

type MockDeps = OwnedDeps<MockStorage, MockApi, MockQuerier>;

fn mock_init(mut deps: DepsMut, msg: InstantiateMsg) {
    let info = mock_info(TEST_CREATOR, &[]);
    let _res = instantiate(deps, mock_env(), info, msg).unwrap();
}

pub fn execute_as_admin(deps: &mut MockDeps, msg: ExecuteMsg) -> ProxyResult {
    let info = mock_info(TEST_CREATOR, &[]);
    execute(deps.as_mut(), mock_env(), info, msg)
}

/**
 * Tests successful instantiation of the contract.
 * Addition of a dapp
 * Removal of a dapp
 */
#[test]
fn successful_initialization() {
    let mut deps = mock_dependencies();

    let msg = instantiate_msg(0);
    let info = mock_info(TEST_CREATOR, &[]);
    let res = instantiate(deps.as_mut(), mock_env(), info.clone(), msg).unwrap();

    // Response should have 0 msgs
    assert_that(&res.messages).is_empty();

    let state: State = STATE.load(&deps.storage).unwrap();
    assert_that(&state).is_equal_to(&State { modules: vec![] });

    let msg = ExecuteMsg::AddModule {
        module: DAPP.to_string(),
    };
    let _res = execute(deps.as_mut(), mock_env(), info.clone(), msg).unwrap();
    let actual_state: State = STATE.load(&deps.storage).unwrap();

    assert_that(&actual_state).is_equal_to(&State {
        modules: vec![Addr::unchecked(DAPP)],
    });

    let msg = ExecuteMsg::RemoveModule {
        module: DAPP.to_string(),
    };
    let _res = execute(deps.as_mut(), mock_env(), info, msg).unwrap();
    let state: State = STATE.load(&deps.storage).unwrap();
    assert_eq!(state, State { modules: vec![] });
}

/**
 * Tests successful Vault Asset update
 */
#[test]
fn successful_asset_update() {
    let mut deps = mock_dependencies();

    let msg = instantiate_msg(0);
    let info = mock_info(TEST_CREATOR, &[]);
    let res = instantiate(deps.as_mut(), mock_env(), info.clone(), msg).unwrap();
    // Response should have 0 msgs
    assert_eq!(0, res.messages.len());

    let state: State = STATE.load(&deps.storage).unwrap();
    assert_eq!(state, State { modules: vec![] });

    let asset1 = "asset1";
    let proxy_asset_1 = UncheckedProxyAsset::new(asset1, None);

    let asset2 = "asset2";
    let proxy_asset_2 = UncheckedProxyAsset::new(asset2, None);

    let msg = ExecuteMsg::UpdateAssets {
        to_add: vec![proxy_asset_1.clone(), proxy_asset_2.clone()],
        to_remove: vec![],
    };

    let res = execute_as_admin(&mut deps, msg);
    assert_that(&res).is_ok();

    // Get an asset
    let actual_asset_1: ProxyAsset = VAULT_ASSETS.load(&deps.storage, asset1.into()).unwrap();

    assert_that(&actual_asset_1.asset.to_string()).is_equal_to(proxy_asset_1.asset);

    // Get the other asset
    let actual_asset_2: ProxyAsset = VAULT_ASSETS.load(&deps.storage, asset2.into()).unwrap();
    assert_that(&actual_asset_2.asset.to_string()).is_equal_to(proxy_asset_2.asset);

    // // Remove token
    // let msg = ExecuteMsg::UpdateAssets {
    //     to_add: vec![],
    //     to_remove: vec![test_token_asset.asset.info.clone()],
    // };
    //
    // let _res = execute(deps.as_mut(), mock_env(), info, msg).unwrap();
    //
    // let _failed_load = VAULT_ASSETS
    //     .load(
    //         &deps.storage,
    //         &get_asset_identifier(&test_token_asset.asset.info),
    //     )
    //     .unwrap_err();
}
