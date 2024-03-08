#![warn(missing_docs)]
//! # Dex Adapter ANS Action Definition
//!
use abstract_core::objects::{AnsAsset, AssetEntry};
use abstract_sdk::Resolve;

use crate::{
    raw_action::{MoneymarketRawAction, MoneymarketRawRequest},
    MoneymarketCommand,
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
        /// Asset that identifies the market you want to deposit in
        borrowed_asset: AssetEntry,
        /// Asset to deposit
        collateral_asset: AnsAsset,
    },
    /// Deposit Collateral to borrow against
    WithdrawCollateral {
        /// Asset that identifies the market you want to withdraw from
        borrowed_asset: AssetEntry,
        /// Asset to deposit
        collateral_asset: AnsAsset,
    },
    /// Borrow funds from the money market
    Borrow {
        /// Asset to borrow
        borrowed_asset: AnsAsset,
        /// Asset that indentifies the market you want to borrow from
        collateral_asset: AssetEntry,
    },
    /// Repay funds to the money market
    Repay {
        /// Asset to repay
        borrowed_asset: AnsAsset,
        /// Asset that indentifies the market you want to borrow from
        collateral_asset: AssetEntry,
    },
}

/// Structure created to be able to resolve an action using ANS
pub struct WholeMoneymarketAction(pub Box<dyn MoneymarketCommand>, pub MoneymarketAnsAction);

impl Resolve for WholeMoneymarketAction {
    type Output = MoneymarketRawAction;

    /// TODO: this only works for protocols where there is only one address for depositing
    fn resolve(
        &self,
        querier: &cosmwasm_std::QuerierWrapper,
        ans_host: &abstract_sdk::feature_objects::AnsHost,
    ) -> abstract_core::objects::ans_host::AnsHostResult<Self::Output> {
        let raw_action = match self.1.clone() {
            MoneymarketAnsAction::Deposit { asset } => {
                let contract_addr =
                    self.0
                        .lending_address(querier, ans_host, asset.name.clone())?;
                let asset = asset.resolve(querier, ans_host)?;
                MoneymarketRawAction {
                    request: MoneymarketRawRequest::Deposit {
                        asset: asset.into(),
                    },
                    contract_addr: contract_addr.to_string(),
                }
            }
            MoneymarketAnsAction::Withdraw { asset } => {
                let contract_addr =
                    self.0
                        .lending_address(querier, ans_host, asset.name.clone())?;
                let asset = asset.resolve(querier, ans_host)?;
                MoneymarketRawAction {
                    request: MoneymarketRawRequest::Withdraw {
                        asset: asset.into(),
                    },
                    contract_addr: contract_addr.to_string(),
                }
            }
            MoneymarketAnsAction::ProvideCollateral {
                borrowed_asset,
                collateral_asset,
            } => {
                let contract_addr = self.0.collateral_address(
                    querier,
                    ans_host,
                    borrowed_asset.clone(),
                    collateral_asset.name.clone(),
                )?;
                let borrowed_asset = borrowed_asset.resolve(querier, ans_host)?;
                let collateral_asset = collateral_asset.resolve(querier, ans_host)?;
                MoneymarketRawAction {
                    request: MoneymarketRawRequest::ProvideCollateral {
                        borrowed_asset: borrowed_asset.into(),
                        collateral_asset: collateral_asset.into(),
                    },
                    contract_addr: contract_addr.to_string(),
                }
            }
            MoneymarketAnsAction::WithdrawCollateral {
                borrowed_asset,
                collateral_asset,
            } => {
                let contract_addr = self.0.collateral_address(
                    querier,
                    ans_host,
                    borrowed_asset.clone(),
                    collateral_asset.name.clone(),
                )?;
                let borrowed_asset = borrowed_asset.resolve(querier, ans_host)?;
                let collateral_asset = collateral_asset.resolve(querier, ans_host)?;
                MoneymarketRawAction {
                    request: MoneymarketRawRequest::WithdrawCollateral {
                        borrowed_asset: borrowed_asset.into(),
                        collateral_asset: collateral_asset.into(),
                    },
                    contract_addr: contract_addr.to_string(),
                }
            }
            MoneymarketAnsAction::Borrow {
                borrowed_asset,
                collateral_asset,
            } => {
                let contract_addr = self.0.borrow_address(
                    querier,
                    ans_host,
                    borrowed_asset.name.clone(),
                    collateral_asset.clone(),
                )?;
                let borrowed_asset = borrowed_asset.resolve(querier, ans_host)?;
                let collateral_asset = collateral_asset.resolve(querier, ans_host)?;
                MoneymarketRawAction {
                    request: MoneymarketRawRequest::Borrow {
                        borrowed_asset: borrowed_asset.into(),
                        collateral_asset: collateral_asset.into(),
                    },
                    contract_addr: contract_addr.to_string(),
                }
            }
            MoneymarketAnsAction::Repay {
                borrowed_asset,
                collateral_asset,
            } => {
                let contract_addr = self.0.borrow_address(
                    querier,
                    ans_host,
                    borrowed_asset.name.clone(),
                    collateral_asset.clone(),
                )?;
                let borrowed_asset = borrowed_asset.resolve(querier, ans_host)?;
                let collateral_asset = collateral_asset.resolve(querier, ans_host)?;
                MoneymarketRawAction {
                    request: MoneymarketRawRequest::Repay {
                        borrowed_asset: borrowed_asset.into(),
                        collateral_asset: collateral_asset.into(),
                    },
                    contract_addr: contract_addr.to_string(),
                }
            }
        };

        Ok(raw_action)
    }
}
