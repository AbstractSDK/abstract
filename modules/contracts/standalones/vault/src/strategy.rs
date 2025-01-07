use std::collections::HashMap;

use cosmwasm_std::{Addr, Coin, CosmosMsg, Deps, QuerierWrapper, StdResult, Uint128};

use crate::state::STRATEGIES;

#[cosmwasm_schema::cw_serde]
pub enum StrategyDescription {
    // Corresponds to not using the funds, they stay in the contract
    Idle,
}

#[cosmwasm_schema::cw_serde]
pub struct Strategy {
    /// percentage correspond to the percentage of total funds that should be invested in this strategy
    pub percentage: f64,

    /// Specification of the strategy to use for this share of the funds
    pub description: StrategyDescription,
}

pub struct AllStrategies {
    pub strategies: Vec<Strategy>,
}

impl AllStrategies {
    pub fn total_invested_value(
        &self,
        querier: &QuerierWrapper,
        price_sources: &HashMap<String, String>,
        this_deposit: &[Coin],
    ) -> Uint128 {
        todo!()
    }
}

impl StrategyDescription {
    pub fn total_invested_value(
        &self,
        querier: &QuerierWrapper,
        price_sources: &HashMap<String, String>,
        this_deposit: &[Coin],
    ) -> Uint128 {
        match self {
            Self::Idle => {
                // Return aggregated current contract balance of all whitelisted funds
                todo!()
            }
        }
    }

    pub fn invest(&self, amount: &[Coin]) -> Vec<CosmosMsg> {
        match self {
            StrategyDescription::Idle => vec![],
        }
    }
}

// We should create a trait so that all strategies have the same behavior
// All nested structs inside the StrategyDescription enum should implement this trait
pub trait TStrategy {
    fn total_invested_value(&self, this_deposit: &[Coin]) -> Uint128;
    // Should invest amount to the current strategy
    fn invest(&self, amount: Uint128) -> Vec<CosmosMsg>;
    // Should withdraw amount from the current strategy and send it to the receiver
    fn withdraw(&self, amount: Uint128, receiver: Addr) -> Vec<CosmosMsg>;
    // Should return the total amount of funds that are invested in the current strategy
    fn total_invested_funds(&self) -> Uint128;
    // Future functions needed will be available here
}
