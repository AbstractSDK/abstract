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
        MoneymarketName, MONEYMARKET_BORROWING_CONTRACT, MONEYMARKET_COLLATERAL_CONTRACT,
        MONEYMARKET_LENDING_CONTRACT,
    },
    raw_action::{MoneymarketRawAction, MoneymarketRawRequest},
};

/// Possible actions to perform on a Money Market
/// This is an example using raw assets
#[cosmwasm_schema::cw_serde]
pub enum MoneymarketAnsAction {
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
pub struct WholeMoneymarketAction(pub MoneymarketName, pub MoneymarketAnsAction);

/// Returns the first pool address to be able to swap given assets on the given dex
pub fn pool_address(
    dex: MoneymarketName,
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

impl Resolve for WholeMoneymarketAction {
    type Output = MoneymarketRawAction;

    /// TODO: this only works for protocols where there is only one address for depositing
    fn resolve(
        &self,
        querier: &cosmwasm_std::QuerierWrapper,
        ans_host: &abstract_sdk::feature_objects::AnsHost,
    ) -> abstract_core::objects::ans_host::AnsHostResult<Self::Output> {
        let (contract_type, asset) = match self.1.clone() {
            MoneymarketAnsAction::Deposit { asset } => (MONEYMARKET_LENDING_CONTRACT, asset),
            MoneymarketAnsAction::Withdraw { asset } => (MONEYMARKET_LENDING_CONTRACT, asset),
            MoneymarketAnsAction::ProvideCollateral { asset } => {
                (MONEYMARKET_COLLATERAL_CONTRACT, asset)
            }
            MoneymarketAnsAction::WithdrawCollateral { asset } => {
                (MONEYMARKET_COLLATERAL_CONTRACT, asset)
            }
            MoneymarketAnsAction::Borrow { asset } => (MONEYMARKET_BORROWING_CONTRACT, asset),
            MoneymarketAnsAction::Repay { asset } => (MONEYMARKET_BORROWING_CONTRACT, asset),
        };

        let raw_asset = asset.resolve(querier, ans_host)?;
        let contract_addr = ContractEntry {
            protocol: self.0.clone(),
            contract: contract_type.to_string(),
        }
        .resolve(querier, ans_host)?;

        Ok(MoneymarketRawAction {
            request: match &self.1 {
                MoneymarketAnsAction::Deposit { .. } => MoneymarketRawRequest::Deposit {
                    asset: raw_asset.into(),
                },
                MoneymarketAnsAction::Withdraw { .. } => MoneymarketRawRequest::Withdraw {
                    asset: raw_asset.into(),
                },
                MoneymarketAnsAction::ProvideCollateral { .. } => {
                    MoneymarketRawRequest::ProvideCollateral {
                        asset: raw_asset.into(),
                    }
                }
                MoneymarketAnsAction::WithdrawCollateral { .. } => {
                    MoneymarketRawRequest::WithdrawCollateral {
                        asset: raw_asset.into(),
                    }
                }
                MoneymarketAnsAction::Borrow { .. } => MoneymarketRawRequest::Borrow {
                    asset: raw_asset.into(),
                },
                MoneymarketAnsAction::Repay { .. } => MoneymarketRawRequest::Repay {
                    asset: raw_asset.into(),
                },
            },
            contract_addr: contract_addr.to_string(),
        })
    }
}
