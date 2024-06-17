#![warn(missing_docs)]
//! # Dex Adapter ANS Action Definition
//!
use abstract_sdk::Resolve;
use abstract_std::objects::{AnsAsset, AssetEntry};

use crate::{
    raw_action::{MoneyMarketRawAction, MoneyMarketRawRequest},
    MoneyMarketCommand,
};

/// Possible actions to perform on a Money Market
/// The following actions use the Abstarct Name Service
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
        lent_asset: AnsAsset,
    },
    /// Deposit Collateral to borrow against
    ProvideCollateral {
        /// Asset that identifies the market you want to deposit in
        borrowable_asset: AssetEntry,
        /// Asset to deposit
        collateral_asset: AnsAsset,
    },
    /// Withdraw Collateral to borrow against
    WithdrawCollateral {
        /// Asset that identifies the market you want to withdraw from
        borrowable_asset: AssetEntry,
        /// Asset to deposit
        collateral_asset: AnsAsset,
    },
    /// Borrow funds from the money market
    Borrow {
        /// Asset to borrow
        borrow_asset: AnsAsset,
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
pub struct MoneyMarketActionResolveWrapper(
    pub Box<dyn MoneyMarketCommand>,
    pub MoneyMarketAnsAction,
);

impl Resolve for MoneyMarketActionResolveWrapper {
    type Output = MoneyMarketRawAction;

    /// TODO: this only works for protocols where there is only one address for depositing
    fn resolve(
        &self,
        querier: &cosmwasm_std::QuerierWrapper,
        ans_host: &abstract_sdk::feature_objects::AnsHost,
    ) -> abstract_std::objects::ans_host::AnsHostResult<Self::Output> {
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
            MoneyMarketAnsAction::Withdraw { lent_asset } => {
                let contract_addr =
                    self.0
                        .lending_address(querier, ans_host, lent_asset.name.clone())?;

                let lent_asset = lent_asset.resolve(querier, ans_host)?;
                MoneyMarketRawAction {
                    request: MoneyMarketRawRequest::Withdraw {
                        lent_asset: lent_asset.into(),
                    },
                    contract_addr: contract_addr.to_string(),
                }
            }
            MoneyMarketAnsAction::ProvideCollateral {
                borrowable_asset,
                collateral_asset,
            } => {
                let contract_addr = self.0.collateral_address(
                    querier,
                    ans_host,
                    borrowable_asset.clone(),
                    collateral_asset.name.clone(),
                )?;
                let borrowable_asset = borrowable_asset.resolve(querier, ans_host)?;
                let collateral_asset = collateral_asset.resolve(querier, ans_host)?;
                MoneyMarketRawAction {
                    request: MoneyMarketRawRequest::ProvideCollateral {
                        borrowable_asset: borrowable_asset.into(),
                        collateral_asset: collateral_asset.into(),
                    },
                    contract_addr: contract_addr.to_string(),
                }
            }
            MoneyMarketAnsAction::WithdrawCollateral {
                borrowable_asset,
                collateral_asset,
            } => {
                let contract_addr = self.0.collateral_address(
                    querier,
                    ans_host,
                    borrowable_asset.clone(),
                    collateral_asset.name.clone(),
                )?;
                let borrowable_asset = borrowable_asset.resolve(querier, ans_host)?;
                let collateral_asset = collateral_asset.resolve(querier, ans_host)?;
                MoneyMarketRawAction {
                    request: MoneyMarketRawRequest::WithdrawCollateral {
                        borrowable_asset: borrowable_asset.into(),
                        collateral_asset: collateral_asset.into(),
                    },
                    contract_addr: contract_addr.to_string(),
                }
            }
            MoneyMarketAnsAction::Borrow {
                borrow_asset,
                collateral_asset,
            } => {
                let contract_addr = self.0.borrow_address(
                    querier,
                    ans_host,
                    borrow_asset.name.clone(),
                    collateral_asset.clone(),
                )?;
                let borrow_asset = borrow_asset.resolve(querier, ans_host)?;
                let collateral_asset = collateral_asset.resolve(querier, ans_host)?;
                MoneyMarketRawAction {
                    request: MoneyMarketRawRequest::Borrow {
                        borrow_asset: borrow_asset.into(),
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
