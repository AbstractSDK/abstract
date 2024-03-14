use cosmwasm_std::{Decimal, Uint128};

pub use ans::{MoneyMarketAnsQuery, WholeMoneyMarketQuery};
pub use raw::MoneyMarketRawQuery;

mod raw {
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
            contract_addr: String,
        },
        /// Deposited Collateral funds
        UserCollateral {
            /// User that has deposited some collateral
            user: String,
            /// Collateral asset to query
            collateral_asset: AssetInfoBase<String>,
            /// Borrowed asset to query
            borrowed_asset: AssetInfoBase<String>,
            contract_addr: String,
        },
        /// Borrowed funds
        UserBorrow {
            /// User that has borrowed some funds
            user: String,
            /// Collateral asset to query
            collateral_asset: AssetInfoBase<String>,
            /// Borrowed asset to query
            borrowed_asset: AssetInfoBase<String>,
            contract_addr: String,
        },
        /// Current Loan-to-Value ratio
        /// Represents the borrow usage for a specific user
        /// Allows to know how much asset are currently borrowed
        CurrentLTV {
            /// User that has borrowed some funds
            user: String,
            /// Collateral asset to query
            collateral_asset: AssetInfoBase<String>,
            /// Borrowed asset to query
            borrowed_asset: AssetInfoBase<String>,
            contract_addr: String,
        },
        /// Maximum Loan to Value ratio for a user
        /// Allows to know how much assets can to be borrowed
        MaxLTV {
            /// User that has borrowed some funds
            user: String,
            /// Collateral asset to query
            collateral_asset: AssetInfoBase<String>,
            /// Borrowed asset to query
            borrowed_asset: AssetInfoBase<String>,
            contract_addr: String,
        },
        /// Price of an asset compared to another asset
        /// The returned decimal corresponds to
        /// How much quote assets can be bought with 1 base asset
        Price {
            quote: AssetInfoBase<String>,
            base: AssetInfoBase<String>,
        },
    }
}

mod ans {
    use crate::MoneyMarketCommand;
    use abstract_core::objects::AssetEntry;
    use abstract_sdk::Resolve;

    use super::raw::MoneyMarketRawQuery;

    /// Possible ans queries to run on the Money Market
    #[cosmwasm_schema::cw_serde]
    pub enum MoneyMarketAnsQuery {
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
            collateral_asset: AssetEntry,
            /// Borrowed asset to query
            borrowed_asset: AssetEntry,
        },
        /// Borrowed funds
        UserBorrow {
            /// User that has borrowed some funds
            user: String,
            /// Collateral asset to query
            collateral_asset: AssetEntry,
            /// Borrowed asset to query
            borrowed_asset: AssetEntry,
        },
        /// Current Loan-to-Value ratio
        /// Represents the borrow usage for a specific user
        /// Allows to know how much asset are currently borrowed
        CurrentLTV {
            /// User that has borrowed some funds
            user: String,
            /// Collateral asset to query
            collateral_asset: AssetEntry,
            /// Borrowed asset to query
            borrowed_asset: AssetEntry,
        },
        /// Maximum Loan to Value ratio for a user
        /// Allows to know how much assets can to be borrowed
        MaxLTV {
            /// User that has borrowed some funds
            user: String,
            /// Collateral asset to query
            collateral_asset: AssetEntry,
            /// Borrowed asset to query
            borrowed_asset: AssetEntry,
        },
        /// Price of an asset compared to another asset
        /// The returned decimal corresponds to
        /// How much quote assets can be bought with 1 base asset
        Price { quote: AssetEntry, base: AssetEntry },
    }

    /// Structure created to be able to resolve an action using ANS
    pub struct WholeMoneyMarketQuery(pub Box<dyn MoneyMarketCommand>, pub MoneyMarketAnsQuery);

    impl Resolve for WholeMoneyMarketQuery {
        type Output = MoneyMarketRawQuery;

        /// TODO: this only works for protocols where there is only one address for depositing
        fn resolve(
            &self,
            querier: &cosmwasm_std::QuerierWrapper,
            ans_host: &abstract_sdk::feature_objects::AnsHost,
        ) -> abstract_core::objects::ans_host::AnsHostResult<Self::Output> {
            let raw_action = match self.1.clone() {
                MoneyMarketAnsQuery::UserDeposit { user, asset } => {
                    let contract_addr = self.0.lending_address(querier, ans_host, asset.clone())?;
                    let asset = asset.resolve(querier, ans_host)?;
                    MoneyMarketRawQuery::UserDeposit {
                        asset: asset.into(),
                        user,
                        contract_addr: contract_addr.to_string(),
                    }
                }
                MoneyMarketAnsQuery::UserCollateral {
                    user,
                    collateral_asset,
                    borrowed_asset,
                } => {
                    let contract_addr = self.0.collateral_address(
                        querier,
                        ans_host,
                        borrowed_asset.clone(),
                        collateral_asset.clone(),
                    )?;
                    let collateral_asset = collateral_asset.resolve(querier, ans_host)?;
                    let borrowed_asset = borrowed_asset.resolve(querier, ans_host)?;
                    MoneyMarketRawQuery::UserCollateral {
                        user,
                        collateral_asset: collateral_asset.into(),
                        borrowed_asset: borrowed_asset.into(),
                        contract_addr: contract_addr.to_string(),
                    }
                }
                MoneyMarketAnsQuery::UserBorrow {
                    user,
                    collateral_asset,
                    borrowed_asset,
                } => {
                    let contract_addr = self.0.borrow_address(
                        querier,
                        ans_host,
                        borrowed_asset.clone(),
                        collateral_asset.clone(),
                    )?;
                    let collateral_asset = collateral_asset.resolve(querier, ans_host)?;
                    let borrowed_asset = borrowed_asset.resolve(querier, ans_host)?;
                    MoneyMarketRawQuery::UserBorrow {
                        user,
                        collateral_asset: collateral_asset.into(),
                        borrowed_asset: borrowed_asset.into(),

                        contract_addr: contract_addr.to_string(),
                    }
                }
                MoneyMarketAnsQuery::CurrentLTV {
                    user,
                    collateral_asset,
                    borrowed_asset,
                } => {
                    let contract_addr = self.0.current_ltv_address(
                        querier,
                        ans_host,
                        borrowed_asset.clone(),
                        collateral_asset.clone(),
                    )?;
                    let collateral_asset = collateral_asset.resolve(querier, ans_host)?;
                    let borrowed_asset = borrowed_asset.resolve(querier, ans_host)?;
                    MoneyMarketRawQuery::CurrentLTV {
                        user,
                        collateral_asset: collateral_asset.into(),
                        borrowed_asset: borrowed_asset.into(),

                        contract_addr: contract_addr.to_string(),
                    }
                }
                MoneyMarketAnsQuery::MaxLTV {
                    user,
                    collateral_asset,
                    borrowed_asset,
                } => {
                    let contract_addr = self.0.max_ltv_address(
                        querier,
                        ans_host,
                        borrowed_asset.clone(),
                        collateral_asset.clone(),
                    )?;
                    let collateral_asset = collateral_asset.resolve(querier, ans_host)?;
                    let borrowed_asset = borrowed_asset.resolve(querier, ans_host)?;
                    MoneyMarketRawQuery::MaxLTV {
                        user,
                        collateral_asset: collateral_asset.into(),
                        borrowed_asset: borrowed_asset.into(),

                        contract_addr: contract_addr.to_string(),
                    }
                }
                MoneyMarketAnsQuery::Price { quote, base } => {
                    let quote = quote.resolve(querier, ans_host)?;
                    let base = base.resolve(querier, ans_host)?;
                    MoneyMarketRawQuery::Price {
                        quote: quote.into(),
                        base: base.into(),
                    }
                }
            };

            Ok(raw_action)
        }
    }
}

/// Responses for MoneyMarketQueries
#[cosmwasm_schema::cw_serde]
pub enum MoneyMarketQueryResponse {
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
