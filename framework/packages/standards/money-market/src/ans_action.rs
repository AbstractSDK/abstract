#![warn(missing_docs)]
//! # Dex Adapter ANS Action Definition
//!
use abstract_core::objects::{AnsAsset, AssetEntry};
use abstract_sdk::Resolve;

use crate::{
    raw_action::{MoneyMarketRawAction, MoneyMarketRawRequest},
    MoneyMarketCommand,
};

/// Possible actions to perform on a Money Market
/// This is an example using raw assets
#[cosmwasm_schema::cw_serde]
pub enum MoneyMarketAnsAction {
    /// Deposit funds for lending.
    Deposit {
        /// Asset to deposit
        lending_asset: AnsAsset,
    },
    /// Withdraw lent funds
    Withdraw {
        /// Asset to withdraw
        lending_asset: AnsAsset,
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
        /// Asset that identifies the market you want to borrow from
        collateral_asset: AssetEntry,
    },
    /// Repay funds to the money market
    Repay {
        /// Asset to repay
        borrowed_asset: AnsAsset,
        /// Asset that identifies the market you want to borrow from
        collateral_asset: AssetEntry,
    },
}

/// Structure created to be able to resolve an action using ANS
pub struct WholeMoneyMarketAction(pub Box<dyn MoneyMarketCommand>, pub MoneyMarketAnsAction);

impl Resolve for WholeMoneyMarketAction {
    type Output = MoneyMarketRawAction;

    /// TODO: this only works for protocols where there is only one address for depositing
    fn resolve(
        &self,
        querier: &cosmwasm_std::QuerierWrapper,
        ans_host: &abstract_sdk::feature_objects::AnsHost,
    ) -> abstract_core::objects::ans_host::AnsHostResult<Self::Output> {
        let raw_action = match self.1.clone() {
            MoneyMarketAnsAction::Deposit { lending_asset } => {
                let contract_addr =
                    self.0
                        .lending_address(querier, ans_host, lending_asset.name.clone())?;
                let asset = lending_asset.resolve(querier, ans_host)?;
                MoneyMarketRawAction {
                    request: MoneyMarketRawRequest::Deposit {
                        lending_asset: asset.into(),
                    },
                    contract_addr: contract_addr.to_string(),
                }
            }
            MoneyMarketAnsAction::Withdraw { lending_asset } => {
                let contract_addr =
                    self.0
                        .lending_address(querier, ans_host, lending_asset.name.clone())?;

                let lending_asset = lending_asset.resolve(querier, ans_host)?;
                MoneyMarketRawAction {
                    request: MoneyMarketRawRequest::Withdraw {
                        lending_asset: lending_asset.into(),
                    },
                    contract_addr: contract_addr.to_string(),
                }
            }
            MoneyMarketAnsAction::ProvideCollateral {
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
                MoneyMarketRawAction {
                    request: MoneyMarketRawRequest::ProvideCollateral {
                        borrowed_asset: borrowed_asset.into(),
                        collateral_asset: collateral_asset.into(),
                    },
                    contract_addr: contract_addr.to_string(),
                }
            }
            MoneyMarketAnsAction::WithdrawCollateral {
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
                MoneyMarketRawAction {
                    request: MoneyMarketRawRequest::WithdrawCollateral {
                        borrowed_asset: borrowed_asset.into(),
                        collateral_asset: collateral_asset.into(),
                    },
                    contract_addr: contract_addr.to_string(),
                }
            }
            MoneyMarketAnsAction::Borrow {
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
                MoneyMarketRawAction {
                    request: MoneyMarketRawRequest::Borrow {
                        borrowed_asset: borrowed_asset.into(),
                        collateral_asset: collateral_asset.into(),
                    },
                    contract_addr: contract_addr.to_string(),
                }
            }
            MoneyMarketAnsAction::Repay {
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
                MoneyMarketRawAction {
                    request: MoneyMarketRawRequest::Repay {
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
