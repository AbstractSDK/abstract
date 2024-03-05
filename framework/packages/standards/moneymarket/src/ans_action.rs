#![warn(missing_docs)]
//! # Dex Adapter ANS Action Definition
//!
use abstract_core::objects::{
    ans_host::AnsHostError, AnsAsset, AssetEntry, ContractEntry, DexAssetPairing, PoolAddress,
    PoolReference,
};
use abstract_sdk::{feature_objects::AnsHost, Resolve};

use crate::{
    msg::{
        DexName, MONEYMARKET_BORROWING_CONTRACT, MONEYMARKET_COLLATERAL_CONTRACT,
        MONEYMARKET_LENDING_CONTRACT,
    },
    raw_action::{MoneyMarketRawAction, MoneyMarketRawRequest},
};

/// Possible actions to perform on a Money Market
/// This is an example using raw assets
#[cosmwasm_schema::cw_serde]
pub enum MoneyMarketAnsAction {
    /// Deposit funds for lending.
    Deposit {
        /// Asset to deposit
        asset: AnsAsset,
    },
    /// Withdraw lended funds
    Withdraw {
        /// Asset to withdraw
        asset: AnsAsset,
    },
    /// Deposit Collateral to borrow against
    ProvideCollateral {
        /// Asset to deposit
        asset: AnsAsset,
    },
    /// Deposit Collateral to borrow against
    WithdrawCollateral {
        /// Asset to deposit
        asset: AnsAsset,
    },
    /// Borrow funds from the money market
    Borrow {
        /// Asset to deposit
        asset: AnsAsset,
    },
    /// Repay funds to the money market
    Repay {
        /// Asset to deposit
        asset: AnsAsset,
    },
}

/// Structure created to be able to resolve an action using ANS
pub struct WholeMoneyMarketAction(pub DexName, pub MoneyMarketAnsAction);

/// Returns the first pool address to be able to swap given assets on the given dex
pub fn pool_address(
    dex: DexName,
    assets: (AssetEntry, AssetEntry),
    querier: &cosmwasm_std::QuerierWrapper,
    ans_host: &AnsHost,
) -> abstract_core::objects::ans_host::AnsHostResult<PoolAddress> {
    let dex_pair = DexAssetPairing::new(assets.0, assets.1, &dex);
    let mut pool_ref = ans_host.query_asset_pairing(querier, &dex_pair)?;
    // Currently takes the first pool found, but should be changed to take the best pool
    let found: PoolReference = pool_ref.pop().ok_or(AnsHostError::DexPairingNotFound {
        pairing: dex_pair,
        ans_host: ans_host.address.clone(),
    })?;
    Ok(found.pool_address)
}

impl Resolve for WholeMoneyMarketAction {
    type Output = MoneyMarketRawAction;

    /// TODO: this only works for protocols where there is only one address for depositing
    fn resolve(
        &self,
        querier: &cosmwasm_std::QuerierWrapper,
        ans_host: &abstract_sdk::feature_objects::AnsHost,
    ) -> abstract_core::objects::ans_host::AnsHostResult<Self::Output> {
        let (contract_type, asset) = match self.1.clone() {
            MoneyMarketAnsAction::Deposit { asset } => (MONEYMARKET_LENDING_CONTRACT, asset),
            MoneyMarketAnsAction::Withdraw { asset } => (MONEYMARKET_LENDING_CONTRACT, asset),
            MoneyMarketAnsAction::ProvideCollateral { asset } => {
                (MONEYMARKET_COLLATERAL_CONTRACT, asset)
            }
            MoneyMarketAnsAction::WithdrawCollateral { asset } => {
                (MONEYMARKET_COLLATERAL_CONTRACT, asset)
            }
            MoneyMarketAnsAction::Borrow { asset } => (MONEYMARKET_BORROWING_CONTRACT, asset),
            MoneyMarketAnsAction::Repay { asset } => (MONEYMARKET_BORROWING_CONTRACT, asset),
        };

        let raw_asset = asset.resolve(querier, ans_host)?;
        let contract_addr = ContractEntry {
            protocol: self.0.clone(),
            contract: contract_type.to_string(),
        }
        .resolve(querier, ans_host)?;

        Ok(MoneyMarketRawAction {
            request: match &self.1 {
                MoneyMarketAnsAction::Deposit { .. } => {
                    MoneyMarketRawRequest::Deposit { asset: raw_asset.into() }
                }
                MoneyMarketAnsAction::Withdraw { .. } => {
                    MoneyMarketRawRequest::Withdraw { asset: raw_asset.into() }
                }
                MoneyMarketAnsAction::ProvideCollateral { .. } => {
                    MoneyMarketRawRequest::ProvideCollateral { asset: raw_asset.into() }
                }
                MoneyMarketAnsAction::WithdrawCollateral { .. } => {
                    MoneyMarketRawRequest::WithdrawCollateral { asset: raw_asset.into() }
                }
                MoneyMarketAnsAction::Borrow { .. } => {
                    MoneyMarketRawRequest::Borrow { asset: raw_asset.into() }
                }
                MoneyMarketAnsAction::Repay { .. } => {
                    MoneyMarketRawRequest::Repay { asset: raw_asset.into() }
                }
            },
            contract_addr: contract_addr.to_string(),
        })
    }
}
