use cosmwasm_schema::{cw_serde, QueryResponses};

use crate::{
    asset::{Asset, AssetInfo, AssetInfoValidated, AssetValidated, DecimalAsset},
    factory::{ConfigResponse as FactoryConfigResponse, QueryMsg as FactoryQueryMsg},
    fee_config::FeeConfig,
    oracle::{SamplePeriod, TwapResponse},
    stake::ConverterConfig,
};

use cosmwasm_std::{
    to_binary, Addr, Binary, Decimal, Decimal256, QuerierWrapper, StdError, StdResult, Uint128,
    WasmMsg,
};
use cw20::Cw20ReceiveMsg;

#[cfg(test)]
pub mod mock_querier;

mod error;
mod instantiate;
mod referral;
mod utils;

use crate::factory::PairType;
pub use error::ContractError;
pub use instantiate::*;
pub use referral::*;
pub use utils::*;

/// Decimal precision for TWAP results
pub const TWAP_PRECISION: u8 = 6;

/// This structure stores the main parameters for an Wyndex pair
#[cw_serde]
pub struct PairInfo {
    /// Asset information for the assets in the pool
    pub asset_infos: Vec<AssetInfoValidated>,
    /// Pair contract address
    pub contract_addr: Addr,
    /// Pair LP token address
    pub liquidity_token: Addr,
    /// Staking contract address
    pub staking_addr: Addr,
    /// The pool type (xyk, stableswap etc) available in [`PairType`]
    pub pair_type: PairType,
    /// The fee configuration for the pair
    pub fee_config: FeeConfig,
}

impl PairInfo {
    /// Returns the balance for each asset in the pool.
    ///
    /// * **contract_addr** is pair's pool address.
    pub fn query_pools(
        &self,
        querier: &QuerierWrapper,
        contract_addr: impl Into<String>,
    ) -> StdResult<Vec<AssetValidated>> {
        let contract_addr = contract_addr.into();
        self.asset_infos
            .iter()
            .map(|asset_info| {
                Ok(AssetValidated {
                    info: asset_info.clone(),
                    amount: asset_info.query_balance(querier, &contract_addr)?,
                })
            })
            .collect()
    }

    /// Returns the balance for each asset in the pool in decimal.
    ///
    /// * **contract_addr** is pair's pool address.
    pub fn query_pools_decimal(
        &self,
        querier: &QuerierWrapper,
        contract_addr: impl Into<String>,
    ) -> StdResult<Vec<DecimalAsset>> {
        let contract_addr = contract_addr.into();
        self.asset_infos
            .iter()
            .map(|asset_info| {
                Ok(DecimalAsset {
                    info: asset_info.clone(),
                    amount: Decimal256::from_atomics(
                        asset_info.query_balance(querier, &contract_addr)?,
                        asset_info.decimals(querier)?.into(),
                    )
                    .map_err(|_| StdError::generic_err("Decimal256RangeExceeded"))?,
                })
            })
            .collect()
    }
}

/// This structure describes the parameters used for creating a contract.
#[cw_serde]
pub struct InstantiateMsg {
    /// Information about assets in the pool
    pub asset_infos: Vec<AssetInfo>,
    /// The token contract code ID used for the tokens in the pool
    pub token_code_id: u64,
    /// The factory contract address
    pub factory_addr: String,
    /// Optional binary serialised parameters for custom pool types
    pub init_params: Option<Binary>,
    /// The fees for this pair
    pub fee_config: FeeConfig,
    pub staking_config: StakeConfig,
    /// The block time until which trading is disabled
    pub trading_starts: u64,
    /// Address which can call ExecuteMsg::Freeze
    pub circuit_breaker: Option<String>,
}

impl InstantiateMsg {
    /// Returns an error if the fee config is invalid
    pub fn validate_fees(&self) -> Result<(), ContractError> {
        self.fee_config
            .valid_fee_bps()
            .then_some(())
            .ok_or(ContractError::InvalidFeeBps {})
    }
}

#[cw_serde]
pub struct StakeConfig {
    /// The staking contract code ID
    pub staking_code_id: u64,
    pub tokens_per_power: Uint128,
    pub min_bond: Uint128,
    pub unbonding_periods: Vec<u64>,
    pub max_distributions: u32,
    /// Optional converter configuration for the staking contract
    pub converter: Option<ConverterConfig>,
}

impl StakeConfig {
    /// Call this after instantiating the lp token to get a message to instantiate the staking contract
    pub fn into_init_msg(
        self,
        querier: &QuerierWrapper,
        lp_token_address: String,
        factory_addr: String,
    ) -> StdResult<WasmMsg> {
        // Add factory's owner as owner of staking contract (DAO) to allow migration
        let factory_owner = querier
            .query_wasm_smart::<FactoryConfigResponse>(&factory_addr, &FactoryQueryMsg::Config {})?
            .owner
            .to_string();
        Ok(WasmMsg::Instantiate {
            code_id: self.staking_code_id,
            msg: to_binary(&crate::stake::InstantiateMsg {
                cw20_contract: lp_token_address, // address of LP token
                tokens_per_power: self.tokens_per_power,
                min_bond: self.min_bond,
                unbonding_periods: self.unbonding_periods,
                max_distributions: self.max_distributions,
                admin: Some(factory_addr),
                unbonder: None, // TODO: allow specifying unbonder
                converter: self.converter,
            })?,
            funds: vec![],
            admin: Some(factory_owner),
            label: String::from("Wyndex-Stake"),
        })
    }
}

/// This structure describes the execute messages available in the contract.
#[cw_serde]
pub enum ExecuteMsg {
    /// Receives a message of type [`Cw20ReceiveMsg`]
    Receive(Cw20ReceiveMsg),
    /// ProvideLiquidity allows someone to provide liquidity in the pool
    ProvideLiquidity {
        /// The assets available in the pool
        assets: Vec<Asset>,
        /// The slippage tolerance that allows liquidity provision only if the price in the pool doesn't move too much
        slippage_tolerance: Option<Decimal>,
        /// The receiver of LP tokens
        receiver: Option<String>,
    },
    /// Swap performs a swap in the pool
    Swap {
        offer_asset: Asset,
        ask_asset_info: Option<AssetInfo>,
        belief_price: Option<Decimal>,
        max_spread: Option<Decimal>,
        to: Option<String>,
        /// The address that should receive the referral commission
        referral_address: Option<String>,
        /// The commission for the referral.
        /// This is capped by the configured max commission
        referral_commission: Option<Decimal>,
    },
    /// Update the pair configuration
    UpdateConfig { params: Binary },
    /// Update the fees for this pair
    UpdateFees { fee_config: FeeConfig },
    /// ProposeNewOwner creates a proposal to change contract ownership.
    /// The validity period for the proposal is set in the `expires_in` variable.
    ProposeNewOwner {
        /// Newly proposed contract owner
        owner: String,
        /// The date after which this proposal expires
        expires_in: u64,
    },
    /// DropOwnershipProposal removes the existing offer to change contract ownership.
    DropOwnershipProposal {},
    /// Used to claim contract ownership.
    ClaimOwnership {},
    /// Freeze all but withdraw liquidity, can only be called if a circuit breaker is set through a MigrateMsg
    Freeze { frozen: bool },
}

/// This structure describes a CW20 hook message.
#[cw_serde]
pub enum Cw20HookMsg {
    /// Swap a given amount of asset
    Swap {
        ask_asset_info: Option<AssetInfo>,
        belief_price: Option<Decimal>,
        max_spread: Option<Decimal>,
        to: Option<String>,
        /// The address that should receive the referral commission
        referral_address: Option<String>,
        /// The commission for the referral.
        /// This is capped by and defaulting to the configured max commission
        referral_commission: Option<Decimal>,
    },
    /// Withdraw liquidity from the pool
    WithdrawLiquidity { assets: Vec<Asset> },
}

#[cw_serde]
pub enum MigrateMsg {
    UpdateFreeze {
        frozen: bool,
        // TODO: better name. this may be an address that can set frozen itself
        circuit_breaker: Option<String>,
    },
}

/// This structure describes the query messages available in the contract.
#[cw_serde]
#[derive(QueryResponses)]
pub enum QueryMsg {
    /// Returns information about a pair in an object of type [`super::asset::PairInfo`].
    #[returns(PairInfo)]
    Pair {},
    /// Returns information about a pool in an object of type [`PoolResponse`].
    #[returns(PoolResponse)]
    Pool {},
    /// Returns contract configuration settings in a custom [`ConfigResponse`] structure.
    #[returns(ConfigResponse)]
    Config {},
    /// Returns information about the share of the pool in a vector that contains objects of type [`Asset`].
    #[returns(Vec<AssetValidated>)]
    Share { amount: Uint128 },
    /// Returns information about a swap simulation in a [`SimulationResponse`] object.
    #[returns(SimulationResponse)]
    Simulation {
        offer_asset: Asset,
        ask_asset_info: Option<AssetInfo>,
        /// Whether to simulate referral
        referral: bool,
        /// The commission for the referral. Only used if `referral` is set to `true`.
        /// This is capped by and defaulting to the configured max commission
        referral_commission: Option<Decimal>,
    },
    /// Returns information about cumulative prices in a [`ReverseSimulationResponse`] object.
    #[returns(ReverseSimulationResponse)]
    ReverseSimulation {
        offer_asset_info: Option<AssetInfo>,
        ask_asset: Asset,
        /// Whether to simulate referral
        referral: bool,
        /// The commission for the referral. Only used if `referral` is set to `true`.
        /// This is capped by and defaulting to the configured max commission
        referral_commission: Option<Decimal>,
    },
    /// Returns information about the cumulative prices in a [`CumulativePricesResponse`] object
    #[returns(CumulativePricesResponse)]
    CumulativePrices {},
    /// Returns a price history of the given duration
    #[returns(TwapResponse)]
    Twap {
        duration: SamplePeriod,
        /// duration: Day and start_age: 3 means to start from first checkpoint 3 days ago
        start_age: u32,
        /// end_age: None means count until the current time, end_age: Some(0) means til the last checkpoint, which would be more regular
        end_age: Option<u32>,
    },
    /// Returns current D invariant in as a [`u128`] value
    #[returns(Uint128)]
    QueryComputeD {},
    /// Return current spot price of input in terms of output
    #[returns(SpotPriceResponse)]
    SpotPrice { offer: AssetInfo, ask: AssetInfo },
    /// Returns amount of tokens that can be exchanged such that sport remains <= target_price.
    /// The last token of offer should return target_price of ask.
    /// Returns None if price is already above expected.
    #[returns(SpotPricePredictionResponse)]
    SpotPricePrediction {
        offer: AssetInfo,
        ask: AssetInfo,
        /// The maximum amount of offer to be sold
        max_trade: Uint128,
        /// The lowest spot price any offer token should be sold at
        target_price: Decimal,
        /// The maximum number of iterations used to bisect the space.
        /// (higher numbers gives more accuracy at higher gas cost)
        iterations: u8,
    },
}

/// This struct is used to return a query result with the total amount of LP tokens and assets in a specific pool.
#[cw_serde]
pub struct PoolResponse {
    /// The assets in the pool together with asset amounts
    pub assets: Vec<AssetValidated>,
    /// The total amount of LP tokens currently issued
    pub total_share: Uint128,
}

/// This struct is used to return a query result with the general contract configuration.
#[cw_serde]
pub struct ConfigResponse {
    /// Last timestamp when the cumulative prices in the pool were updated
    pub block_time_last: u64,
    /// The pool's parameters
    pub params: Option<Binary>,
    /// The contract owner
    pub owner: Option<Addr>,
}

/// This structure holds the parameters that are returned from a swap simulation response
#[cw_serde]
pub struct SimulationResponse {
    /// The amount of ask assets returned by the swap (denominated in `ask_asset_info`)
    pub return_amount: Uint128,
    /// The spread used in the swap operation (denominated in `ask_asset_info`)
    pub spread_amount: Uint128,
    /// The amount of fees charged by the transaction (denominated in `ask_asset_info`)
    pub commission_amount: Uint128,
    /// The absolute amount of referral commission (denominated in `offer_asset_info`)
    pub referral_amount: Uint128,
}

/// This structure holds the parameters that are returned from a reverse swap simulation response.
#[cw_serde]
pub struct ReverseSimulationResponse {
    /// The amount of offer assets returned by the reverse swap
    pub offer_amount: Uint128,
    /// The spread used in the swap operation
    pub spread_amount: Uint128,
    /// The amount of fees charged by the transaction
    pub commission_amount: Uint128,
    /// The absolute amount of referral commission (denominated in `offer_asset_info`)
    pub referral_amount: Uint128,
}

/// This structure is used to return a cumulative prices query response.
#[cw_serde]
pub struct CumulativePricesResponse {
    /// The assets in the pool to query
    pub assets: Vec<AssetValidated>,
    /// The total amount of LP tokens currently issued
    pub total_share: Uint128,
    /// The vector contains cumulative prices for each pair of assets in the pool
    pub cumulative_prices: Vec<(AssetInfoValidated, AssetInfoValidated, Uint128)>,
}

/// This structure holds stableswap pool parameters.
#[cw_serde]
pub struct StablePoolParams {
    /// The current stableswap pool amplification
    pub amp: u64,
    /// The contract owner
    pub owner: Option<String>,
    /// Information on LSD, if supported (TODO: always require?)
    pub lsd: Option<LsdInfo>,
}

#[cw_serde]
pub struct LsdInfo {
    /// Which asset is the LSD (and thus has the target_rate)
    pub asset: AssetInfo,

    /// Address of the liquid staking hub contract for this pool.
    /// This is used to get the target value to concentrate liquidity around.
    pub hub: String,

    /// The minimum amount of time in seconds between two target value queries
    pub target_rate_epoch: u64,
}

/// This structure stores a stableswap pool's configuration.
#[cw_serde]
pub struct StablePoolConfig {
    /// The stableswap pool amplification
    pub amp: Decimal,
}

/// This enum stores the options available to start and stop changing a stableswap pool's amplification.
#[cw_serde]
pub enum StablePoolUpdateParams {
    StartChangingAmp { next_amp: u64, next_amp_time: u64 },
    StopChangingAmp {},
}

/// This structure holds the parameters that are returned from a reverse swap simulation response.
#[cw_serde]
pub struct SpotPriceResponse {
    pub price: Decimal,
}

#[cw_serde]
pub struct SpotPricePredictionResponse {
    /// Represents units to buy until spot price hits target (in query).
    /// Returns None, result is already below the spot price
    pub trade: Option<Uint128>,
}
