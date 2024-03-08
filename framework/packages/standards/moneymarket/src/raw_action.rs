#![warn(missing_docs)]
//! # Dex Adapter Raw Action Definition

use cw_asset::AssetBase;

/// Possible actions to perform on a Money Market
/// This is an example using raw assets
#[cosmwasm_schema::cw_serde]
pub enum MoneymarketRawRequest {
    /// Deposit funds for lending.
    Deposit {
        /// Asset to deposit
        asset: AssetBase<String>,
    },
    /// Withdraw lended funds
    Withdraw {
        /// Asset to withdraw
        asset: AssetBase<String>,
    },
    /// Deposit Collateral to borrow against
    ProvideCollateral {
        /// Asset to deposit
        asset: AssetBase<String>,
    },
    /// Deposit Collateral to borrow against
    WithdrawCollateral {
        /// Asset to deposit
        asset: AssetBase<String>,
    },
    /// Borrow funds from the money market
    Borrow {
        /// Asset to deposit
        asset: AssetBase<String>,
    },
    /// Repay funds to the money market
    Repay {
        /// Asset to deposit
        asset: AssetBase<String>,
    },
}

/// Action to execute on a moneymarket
#[cosmwasm_schema::cw_serde]
pub struct MoneymarketRawAction {
    /// The action to execute
    pub request: MoneymarketRawRequest,
    /// The contract address to execute it against
    pub contract_addr: String,
}
