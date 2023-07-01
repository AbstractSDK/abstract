use cosmwasm_schema::cw_serde;
use cosmwasm_std::Uint128;

/// Unbonding period in seconds
pub type UnbondingPeriod = u64;

#[cw_serde]
pub struct InstantiateMsg {
    /// address of cw20 contract token
    pub cw20_contract: String,
    pub tokens_per_power: Uint128,
    pub min_bond: Uint128,
    pub unbonding_periods: Vec<UnbondingPeriod>,
    /// the maximum number of distributions that can be created
    pub max_distributions: u32,

    // admin can only add/remove hooks and add distributions, not change other parameters
    pub admin: Option<String>,
    /// Address of the account that can call [`ExecuteMsg::QuickUnbond`]
    pub unbonder: Option<String>,
    /// Configuration for the [`crate::msg::ExecuteMsg::MigrateStake`] message.
    /// Allows converting staked LP tokens to LP tokens of another pool.
    /// E.g. LP tokens of the USDC-JUNO pool can be converted to LP tokens of the USDC-wyJUNO pool
    pub converter: Option<ConverterConfig>,
}

#[cw_serde]
pub struct ConverterConfig {
    /// Address of the contract that converts the LP tokens
    pub contract: String,
    /// Address of the pair contract the converter should convert to
    pub pair_to: String,
}

#[cw_serde]
pub enum ReceiveMsg {
    Delegate {
        /// Unbonding period in seconds
        unbonding_period: u64,
        /// If set, the staked assets will be assigned to the given address instead of the sender
        delegate_as: Option<String>,
    },
    /// This will delegate a large sum on behalf of many different users.
    /// The total amount in delegate_to must be <= the amount of tokens sent.
    /// If it is less, any remainder is staked on behalf of the sender
    MassDelegate {
        /// Unbonding period in seconds
        unbonding_period: u64,
        delegate_to: Vec<(String, Uint128)>,
    },
    /// Fund a distribution flow with cw20 tokens and update the Reward Config for that cw20 asset.
    Fund { funding_info: FundingInfo },
}

#[cw_serde]
pub struct FundingInfo {
    /// Epoch in seconds when distribution should start.
    pub start_time: u64,
    /// Duration of distribution in seconds.
    pub distribution_duration: u64,
    /// Amount to distribute.
    pub amount: Uint128,
}
