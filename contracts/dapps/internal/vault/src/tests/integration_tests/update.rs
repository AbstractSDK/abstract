use cosmwasm_std::{Addr, Uint128};

use terra_multi_test::App;

use crate::msg::ExecuteMsg;

use terra_multi_test::Executor;
use terraswap::asset::Asset;

use pandora_os::core::treasury::msg as TreasuryMsg;
use pandora_os::core::treasury::vault_assets::{ValueRef, VaultAsset};

use super::common_integration::BaseContracts;

pub fn change_base_to_whale(
    app: &mut App,
    owner: Addr,
    base_contracts: &BaseContracts,
    vault_dapp: &Addr,
) {
    // Change base so UST gets valueref to whale
    app.execute_contract(
        owner.clone(),
        base_contracts.treasury.clone(),
        &TreasuryMsg::ExecuteMsg::UpdateAssets {
            to_add: vec![
                // uusd is base asset of this vault, so no value_ref
                VaultAsset {
                    asset: Asset {
                        info: terraswap::asset::AssetInfo::NativeToken {
                            denom: "uusd".to_string(),
                        },
                        amount: Uint128::zero(),
                    },
                    value_reference: Some(ValueRef::Pool {
                        pair_address: base_contracts.whale_ust_pair.clone(),
                    }),
                },
                // Other asset is WHALE. It's value in uusd is calculated with the provided pool valueref
                VaultAsset {
                    asset: Asset {
                        info: terraswap::asset::AssetInfo::Token {
                            contract_addr: base_contracts.whale.to_string(),
                        },
                        amount: Uint128::zero(),
                    },
                    value_reference: None,
                },
            ],
            to_remove: vec![],
        },
        &[],
    )
    .unwrap();

    // Change deposit asset to WHALE
    app.execute_contract(
        owner.clone(),
        vault_dapp.clone(),
        &ExecuteMsg::UpdatePool {
            deposit_asset: Some("whale".to_string()),
            assets_to_add: vec![],
            assets_to_remove: vec![],
        },
        &[],
    )
    .unwrap();
}
