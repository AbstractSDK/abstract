use abstract_core::objects::AssetEntry;
use cosmwasm_std::{Decimal, Uint128};
use cw_asset::AssetInfoBase;

/// Possible raw queries to run on the Money Market
#[cosmwasm_schema::cw_serde]
pub enum MoneymarketRawQuery {
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

/// Possible ans queries to run on the Money Market
#[cosmwasm_schema::cw_serde]
pub enum MoneymarketAnsQuery {
    /// Deposited funds for lending
    UserDeposit {
        /// User that has deposited some funds
        user: String,
        /// Lended asset to query
        asset: AssetEntry,
    },
    /// Deposited Collateral funds
    UserCollateral {
        /// User that has deposited some collateral
        user: String,
        /// Collateral asset to query
        asset: AssetEntry,
    },
    /// Borrowed funds
    UserBorrow {
        /// User that has borrowed some funds
        user: String,
        /// Borrowed asset to query
        asset: AssetEntry,
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
    Price { quote: AssetEntry, base: AssetEntry },
}

/// Responses for MoneyMarketQueries
#[cosmwasm_schema::cw_serde]
pub enum MoneymarketQueryResponse {
    /// Deposited funds for lending
    UserDeposit { deposit_amount: Uint128 },
    /// Deposited Collateral funds
    UserCollateral { provided_collateral_amount: Uint128 },
    /// Borrowed funds
    UserBorrow { borrowed_amount: Uint128 },
    /// Current Loan-to-Value ratio
    /// Represents the borrow usage for a specific user
    /// Allows to know how much asset are currently borrowed
    CurrentLTV { ltv: Decimal },
    /// Maximum Loan to Value ratio for a user
    /// Allows to know how much assets can to be borrowed
    MaxLTV { ltv: Decimal },
    /// Price of an asset compared to another asset
    /// The returned decimal corresponds to
    /// How much quote assets can be bought with 1 base asset
    Price { price: Decimal },
}

pub struct UserMoneyMarketPosition {}
