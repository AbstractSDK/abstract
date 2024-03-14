use cosmwasm_std::{Decimal, Uint128};

pub use ans::{MoneymarketAnsQuery, WholeMoneymarketQuery};
pub use raw::MoneymarketRawQuery;

mod raw {
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
    use crate::MoneymarketCommand;
    use abstract_core::objects::AssetEntry;
    use abstract_sdk::Resolve;

    use super::raw::MoneymarketRawQuery;

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
    pub struct WholeMoneymarketQuery(pub Box<dyn MoneymarketCommand>, pub MoneymarketAnsQuery);

    impl Resolve for WholeMoneymarketQuery {
        type Output = MoneymarketRawQuery;

        /// TODO: this only works for protocols where there is only one address for depositing
        fn resolve(
            &self,
            querier: &cosmwasm_std::QuerierWrapper,
            ans_host: &abstract_sdk::feature_objects::AnsHost,
        ) -> abstract_core::objects::ans_host::AnsHostResult<Self::Output> {
            let raw_action = match self.1.clone() {
                MoneymarketAnsQuery::UserDeposit { user, asset } => {
                    let contract_addr = self.0.lending_address(querier, ans_host, asset.clone())?;
                    let asset = asset.resolve(querier, ans_host)?;
                    MoneymarketRawQuery::UserDeposit {
                        asset: asset.into(),
                        user,
                        contract_addr: contract_addr.to_string(),
                    }
                }
                MoneymarketAnsQuery::UserCollateral {
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
                    MoneymarketRawQuery::UserCollateral {
                        user,
                        collateral_asset: collateral_asset.into(),
                        borrowed_asset: borrowed_asset.into(),
                        contract_addr: contract_addr.to_string(),
                    }
                }
                MoneymarketAnsQuery::UserBorrow {
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
                    MoneymarketRawQuery::UserBorrow {
                        user,
                        collateral_asset: collateral_asset.into(),
                        borrowed_asset: borrowed_asset.into(),

                        contract_addr: contract_addr.to_string(),
                    }
                }
                MoneymarketAnsQuery::CurrentLTV {
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
                    MoneymarketRawQuery::CurrentLTV {
                        user,
                        collateral_asset: collateral_asset.into(),
                        borrowed_asset: borrowed_asset.into(),

                        contract_addr: contract_addr.to_string(),
                    }
                }
                MoneymarketAnsQuery::MaxLTV {
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
                    MoneymarketRawQuery::MaxLTV {
                        user,
                        collateral_asset: collateral_asset.into(),
                        borrowed_asset: borrowed_asset.into(),

                        contract_addr: contract_addr.to_string(),
                    }
                }
                MoneymarketAnsQuery::Price { quote, base } => {
                    let quote = quote.resolve(querier, ans_host)?;
                    let base = base.resolve(querier, ans_host)?;
                    MoneymarketRawQuery::Price {
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
