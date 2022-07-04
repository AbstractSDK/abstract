use cosmwasm_std::{Addr, Coin, Decimal, Uint128};

use cw_multi_test::{App, ContractWrapper};

use crate::{ExecuteMsg, InstantiateMsg, QueryMsg, StateResponse};
use crate::tests::integration_tests::common_integration::{mint_some_whale, store_token_code};
use cw_multi_test::Executor;
use cw_asset::Asset;

use abstract_os::memory::msg as MemoryMsg;
use abstract_os::proxy::msg as TreasuryMsg;
use abstract_os::objects::proxy_assets::{ValueRef, ProxyAsset};

use abstract_os::proxy::dapp_base::BaseInstantiateMsg;

use super::common_integration::{whitelist_dapp, BaseContracts};
const MILLION: u64 = 1_000_000u64;

pub fn init_vault_dapp(app: &mut App, owner: Addr, base_contracts: &BaseContracts) -> (Addr, Addr) {
    // Upload Vault DApp Contract
    let vault_dapp_contract = Box::new(
        ContractWrapper::new_with_empty(
            crate::contract::execute,
            crate::contract::instantiate,
            crate::contract::query,
        )
        .with_reply(crate::contract::reply),
    );

    let vault_dapp_code_id = app.store_code(vault_dapp_contract);
    let lp_contract_code_id = store_token_code(app);

    let vault_dapp_instantiate_msg = InstantiateMsg {
        base: BaseInstantiateMsg {
            trader: owner.to_string(),
            proxy_address: base_contracts.proxy.to_string(),
            memory_addr: base_contracts.memory.to_string(),
        },
        token_code_id: lp_contract_code_id,
        fee: Decimal::percent(10u64),
        deposit_asset: "ust".to_string(),
        vault_lp_token_name: None,
        vault_lp_token_symbol: None,
    };

    // Init contract
    let vault_dapp_instance = app
        .instantiate_contract(
            vault_dapp_code_id,
            owner.clone(),
            &vault_dapp_instantiate_msg,
            &[],
            "vault_dapp",
            None,
        )
        .unwrap();

    // Get liquidity token addr
    let res: StateResponse = app
        .wrap()
        .query_wasm_smart(vault_dapp_instance.clone(), &QueryMsg::State {})
        .unwrap();
    assert_eq!("Contract #6", res.liquidity_token);
    let liquidity_token = res.liquidity_token;

    // Whitelist vault dapp on proxy
    whitelist_dapp(app, &owner, &base_contracts.proxy, &vault_dapp_instance);

    // Add whale with valueref to whale/ust pool
    // Add whale to vault claimable assets.
    app.execute_contract(
        owner.clone(),
        base_contracts.proxy.clone(),
        &TreasuryMsg::ExecuteMsg::UpdateAssets {
            to_add: vec![
                // uusd is base asset of this vault, so no value_ref
                ProxyAsset {
                    asset: Asset {
                        info: cw_asset::AssetInfo::Native(
                            denom: "uusd".to_string(),
                        },
                        amount: Uint128::zero(),
                    },
                    value_reference: None,
                },
                // Other asset is WHALE. It's value in uusd is calculated with the provided pool valueref
                ProxyAsset {
                    asset: Asset {
                        info: cw_asset::AssetInfo::Cw20(
                            contract_addr: base_contracts.whale.to_string(),
                        },
                        amount: Uint128::zero(),
                    },
                    value_reference: Some(ValueRef::Pool {
                        pair_address: base_contracts.whale_ust_pair.clone(),
                    }),
                },
            ],
            to_remove: vec![],
        },
        &[],
    )
    .unwrap();

    // Add whale to vault claimable assets.
    app.execute_contract(
        owner.clone(),
        vault_dapp_instance.clone(),
        &ExecuteMsg::UpdatePool {
            deposit_asset: None,
            assets_to_add: vec!["whale".to_string()],
            assets_to_remove: vec![],
        },
        &[],
    )
    .unwrap();

    // Add uusd and WHALE to whale/ust pool. Price = 0.5 UST/WHALE
    app.init_bank_balance(
        &base_contracts.whale_ust_pair,
        vec![Coin {
            denom: "uusd".to_string(),
            amount: Uint128::from(1_000u64 * MILLION),
        }],
    )
    .unwrap();

    mint_some_whale(
        app,
        owner.clone(),
        base_contracts.whale.clone(),
        Uint128::from(2_000u64 * MILLION),
        base_contracts.whale_ust_pair.to_string(),
    );

    (vault_dapp_instance, Addr::unchecked(liquidity_token))
}

pub fn configure_memory(app: &mut App, sender: Addr, base_contracts: &BaseContracts) {
    // Add whale and whale_ust token to the memory assets
    // Is tested on unit-test level
    app.execute_contract(
        sender.clone(),
        base_contracts.memory.clone(),
        &MemoryMsg::ExecuteMsg::UpdateAssetAddresses {
            to_add: vec![
                (
                    "whale".to_string(),
                    AssetInfoUnchecked::Cw20(base_contracts.whale.to_string()),
                ),
                (
                    "whale_ust".to_string(),
                    AssetInfoUnchecked::Cw20(base_contracts.whale_ust.to_string()),
                ),
                ("ust".to_string(), AssetInfoUnchecked::Native("uusd".to_string())),
            ],
            to_remove: vec![],
        },
        &[],
    )
    .unwrap();

    // Check Memory
    let resp: MemoryMsg::AssetQueryResponse = app
        .wrap()
        .query_wasm_smart(
            &base_contracts.memory,
            &MemoryMsg::QueryMsg::QueryAssets {
                names: vec![
                    "whale".to_string(),
                    "whale_ust".to_string(),
                    "ust".to_string(),
                ],
            },
        )
        .unwrap();

    // Detailed check handled in unit-tests
    assert_eq!("ust".to_string(), resp.assets[0].0);
    assert_eq!("whale".to_string(), resp.assets[1].0);
    assert_eq!("whale_ust".to_string(), resp.assets[2].0);

    // Add whale_ust pair to the memory contracts
    // Is tested on unit-test level
    app.execute_contract(
        sender.clone(),
        base_contracts.memory.clone(),
        &MemoryMsg::ExecuteMsg::UpdateContractAddresses {
            to_add: vec![(
                "whale_ust_pair".to_string(),
                base_contracts.whale_ust_pair.to_string(),
            )],
            to_remove: vec![],
        },
        &[],
    )
    .unwrap();

    // Check Memory
    let resp: MemoryMsg::ContractQueryResponse = app
        .wrap()
        .query_wasm_smart(
            &base_contracts.memory,
            &MemoryMsg::QueryMsg::QueryContracts {
                names: vec!["whale_ust_pair".to_string()],
            },
        )
        .unwrap();

    // Detailed check handled in unit-tests
    assert_eq!("whale_ust_pair".to_string(), resp.contracts[0].0);

    // Check proxy Value
    let proxy_res: TreasuryMsg::TotalValueResponse = app
        .wrap()
        .query_wasm_smart(
            base_contracts.proxy.clone(),
            &TreasuryMsg::QueryMsg::TotalValue {},
        )
        .unwrap();

    assert_eq!(0u128, proxy_res.value.u128());
}
