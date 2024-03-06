use cw_asset::AssetInfoBase;

/// Possible raw queries to run on the Money Market
#[cosmwasm_schema::cw_serde]
pub enum MoneyMarketRawQuery {
    /// Deposited funds for lending
    UserDeposit {
        /// User that has deposited some funds
        user: String,
        /// Lended asset to query
        asset: AssetInfoBase<String>,
    },
    /// Deposited Collateral funds
    UserCollateral {
        /// User that has deposited some collateral
        user: String,
        /// Collateral asset to query
        asset: AssetInfoBase<String>,
    },
    /// Borrowed funds
    UserBorrow {
        /// User that has borrowed some funds
        user: String,
        /// Borrowed asset to query
        asset: AssetInfoBase<String>,
    },
    /// Current Loan-to-Value ratio
    /// Represents the borrow usage for a specific user
    /// Allows to know how much asset are currently borrowed
    CurrentLTV {
        /// User that has borrowed some funds
        user: String,
    },
    /// Maximum Loan to Value ratio for a user
    /// Allows to know how much assets can to be borrowed
    MaxLTV {
        /// User that has borrowed some funds
        user: String,
    },
    /// Price of an asset compared to another asset
    /// The returned decimal corresponds to
    /// How much quote assets can be bought with 1 base asset
    Price {
        quote: AssetInfoBase<String>,
        base: AssetInfoBase<String>,
    },
}

pub struct UserMoneyMarketPosition {}
