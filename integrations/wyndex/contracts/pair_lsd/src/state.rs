use cosmwasm_schema::cw_serde;
use cosmwasm_std::{Addr, Decimal, DepsMut, StdResult, Storage, Uint128};
use cw_storage_plus::{Item, Map};
use wyndex::asset::AssetInfoValidated;
use wyndex::common::OwnershipProposal;
use wyndex::pair::PairInfo;

/// This structure stores the main stableswap pair parameters.
#[cw_serde]
pub struct Config {
    /// The contract owner
    pub owner: Option<Addr>,
    /// The pair information stored in a [`PairInfo`] struct
    pub pair_info: PairInfo,
    /// The factory contract address
    pub factory_addr: Addr,
    /// The last timestamp when the pair contract update the asset cumulative prices
    pub block_time_last: u64,
    /// This is the current amplification used in the pool
    pub init_amp: u64,
    /// This is the start time when amplification starts to scale up or down
    pub init_amp_time: u64,
    /// This is the target amplification to reach at `next_amp_time`
    pub next_amp: u64,
    /// This is the timestamp when the current pool amplification should be `next_amp`
    pub next_amp_time: u64,
    /// The greatest precision of assets in the pool
    pub greatest_precision: u8,
    /// The vector contains cumulative prices for each pair of assets in the pool
    pub cumulative_prices: Vec<(AssetInfoValidated, AssetInfoValidated, Uint128)>,
    /// The block time until which trading is disabled
    pub trading_starts: u64,

    pub lsd: Option<LsdData>,
}

impl Config {
    pub fn target_rate(&self) -> Decimal {
        match &self.lsd {
            Some(lsd) => lsd.target_rate,
            None => Decimal::one(),
        }
    }

    pub fn is_lsd(&self, asset: &AssetInfoValidated) -> bool {
        self.lsd
            .as_ref()
            .map(|l| &l.asset == asset)
            .unwrap_or(false)
    }
}

#[cw_serde]
pub struct LsdData {
    /// Which asset is the LSD (and thus has the target_rate)
    pub asset: AssetInfoValidated,
    /// Address of the liquid staking hub contract for this pool.
    /// This is used to get the target value to concentrate liquidity around.
    pub lsd_hub: Addr,
    /// The target rate to concentrate liquidity around. Defaults to `1.0`.
    /// If `lsd_hub` is set, this is updated once per `target_rate_epoch`.
    pub target_rate: Decimal,
    /// The minimum amount of time in seconds between two target value queries
    pub target_rate_epoch: u64,
    /// The last timestamp when the target value was queried
    pub last_target_query: u64,
}

pub const CONFIG: Item<Config> = Item::new("config");
// Address which can trigger a Freeze or Unfreeze via an ExecuteMsg variant
pub const CIRCUIT_BREAKER: Item<Addr> = Item::new("circuit_breaker");
// Whether the contract is frozen or not
pub const FROZEN: Item<bool> = Item::new("frozen");
/// Stores map of AssetInfo (as String) -> precision
const PRECISIONS: Map<String, u8> = Map::new("precisions");

/// Stores the latest contract ownership transfer proposal
pub const OWNERSHIP_PROPOSAL: Item<OwnershipProposal> = Item::new("ownership_proposal");

/// Store all token precisions and return the greatest one.
pub(crate) fn store_precisions(deps: DepsMut, asset_infos: &[AssetInfoValidated]) -> StdResult<u8> {
    let mut max = 0u8;

    for asset_info in asset_infos {
        let precision = asset_info.decimals(&deps.querier)?;
        max = max.max(precision);
        PRECISIONS.save(deps.storage, asset_info.to_string(), &precision)?;
    }

    Ok(max)
}

/// Loads precision of the given asset info.
pub(crate) fn get_precision(
    storage: &dyn Storage,
    asset_info: &AssetInfoValidated,
) -> StdResult<u8> {
    PRECISIONS.load(storage, asset_info.to_string())
}
