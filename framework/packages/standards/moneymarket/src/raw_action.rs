#![warn(missing_docs)]
//! # Dex Adapter Raw Action Definition

use cw_asset::{AssetBase, AssetInfoBase};

/// Possible actions to perform on a Money Market
/// This is an example using raw assets
#[cosmwasm_schema::cw_serde]
pub enum MoneymarketRawRequest {
    /// Deposit funds for lending.
    Deposit {
        /// Asset to deposit
        lending_asset: AssetBase<String>,
    },
    /// Withdraw lent funds
    Withdraw {
        /// Asset to withdraw
        lending_asset: AssetBase<String>,
    },
    /// Deposit Collateral to borrow against
    ProvideCollateral {
        /// Asset that identifies the market you want to deposit in
        borrowed_asset: AssetInfoBase<String>,
        /// Asset to deposit
        collateral_asset: AssetBase<String>,
    },
    /// Deposit Collateral to borrow against
    WithdrawCollateral {
        /// Asset that identifies the market you want to withdraw from
        borrowed_asset: AssetInfoBase<String>,
        /// Asset to deposit
        collateral_asset: AssetBase<String>,
    },
    /// Borrow funds from the money market
    Borrow {
        /// Asset to borrow
        borrowed_asset: AssetBase<String>,
        /// Asset that identifies the market you want to borrow from
        collateral_asset: AssetInfoBase<String>,
    },
    /// Repay funds to the money market
    Repay {
        /// Asset to repay
        borrowed_asset: AssetBase<String>,
        /// Asset that identifies the market you want to borrow from
        collateral_asset: AssetInfoBase<String>,
    },
}

/// Action to execute on a money_market
#[cosmwasm_schema::cw_serde]
pub struct MoneymarketRawAction {
    /// The action to execute
    pub request: MoneymarketRawRequest,
    /// The contract address to execute it against
    pub contract_addr: String,
}
