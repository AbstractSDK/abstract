use cosmwasm_schema::cw_serde;
use cosmwasm_std::{Addr, Api, Decimal, Deps, Order, StdResult};
use cw_storage_plus::{Bound, Item, Map};
use itertools::Itertools;

use crate::error::ContractError;
use wyndex::asset::{AssetInfo, AssetInfoValidated};
use wyndex::common::OwnershipProposal;
use wyndex::factory::{DefaultStakeConfig, DistributionFlow, PairConfig};

/// This structure holds the main contract parameters.
#[cw_serde]
pub struct Config {
    /// Address allowed to change contract parameters.
    /// This is set to the dao address by default.
    pub owner: Addr,
    /// CW20 token contract code identifier
    pub token_code_id: u64,
    /// Contract address to send governance fees to (the protocol)
    pub fee_address: Option<Addr>,
    /// Maximum referral commission
    pub max_referral_commission: Decimal,
    /// Default values for lp token staking contracts
    pub default_stake_config: DefaultStakeConfig,
    /// When this is set to `true`, only the owner can create pairs
    pub only_owner_can_create_pairs: bool,
    /// The block time until which trading is disabled
    pub trading_starts: Option<u64>,
}

/// This is an intermediate structure for storing a pair's key. It is used in a submessage response.
#[cw_serde]
pub struct TmpPairInfo {
    pub pair_key: Vec<u8>,
    pub asset_infos: Vec<AssetInfoValidated>,
    pub distribution_flows: Vec<DistributionFlow>,
}

/// Saves a pair's key
pub const TMP_PAIR_INFO: Item<TmpPairInfo> = Item::new("tmp_pair_info");

/// Saves factory settings
pub const CONFIG: Item<Config> = Item::new("config");

/// Saves created pairs (from olders to latest)
pub const PAIRS: Map<&[u8], Addr> = Map::new("pair_info");

/// Set of all staking addresses
pub const STAKING_ADDRESSES: Map<&Addr, ()> = Map::new("staking_addresses");

/// Calculates a pair key from the specified parameters in the `asset_infos` variable.
///
/// `asset_infos` is an array with multiple items of type [`AssetInfo`].
pub fn pair_key(asset_infos: &[AssetInfoValidated]) -> Vec<u8> {
    asset_infos
        .iter()
        .map(AssetInfoValidated::as_bytes)
        .sorted()
        .flatten()
        .copied()
        .collect()
}

/// Saves pair type configurations
pub const PAIR_CONFIGS: Map<String, PairConfig> = Map::new("pair_configs");

/// ## Pagination settings
/// The default limit for reading pairs from [`PAIRS`]
const DEFAULT_LIMIT: u32 = 10;

/// Reads pairs from the [`PAIRS`] vector according to the `start_after` and `limit` variables.
/// Otherwise, it returns the default number of pairs, starting from the oldest one.
///
/// `start_after` is the pair from which the function starts to fetch results.
///
/// `limit` is the number of items to retrieve.
pub fn read_pairs(
    deps: Deps,
    start_after: Option<Vec<AssetInfo>>,
    limit: Option<u32>,
) -> StdResult<Vec<Addr>> {
    let start_after = start_after
        .map(|a| {
            a.into_iter()
                .map(|a| a.validate(deps.api))
                .collect::<Result<_, _>>()
        })
        .transpose()?;
    let limit = limit.unwrap_or(DEFAULT_LIMIT) as usize;

    if let Some(start) = calc_range_start(start_after) {
        PAIRS
            .range(
                deps.storage,
                Some(Bound::exclusive(start.as_slice())),
                None,
                Order::Ascending,
            )
            .take(limit)
            .map(|item| {
                let (_, pair_addr) = item?;
                Ok(pair_addr)
            })
            .collect()
    } else {
        PAIRS
            .range(deps.storage, None, None, Order::Ascending)
            .take(limit)
            .map(|item| {
                let (_, pair_addr) = item?;
                Ok(pair_addr)
            })
            .collect()
    }
}

/// Calculates the key of a pair from which to start reading data.
///
/// `start_after` is an [`Option`] type that accepts [`AssetInfo`] elements.
/// It is the token pair which we use to determine the start index for a range when returning data for multiple pairs
fn calc_range_start(start_after: Option<Vec<AssetInfoValidated>>) -> Option<Vec<u8>> {
    start_after.map(|ref asset| {
        let mut key = pair_key(asset);
        key.push(1);
        key
    })
}

pub(crate) fn check_asset_infos(
    api: &dyn Api,
    asset_infos: &[AssetInfo],
) -> Result<Vec<AssetInfoValidated>, ContractError> {
    if !asset_infos.iter().all_unique() {
        return Err(ContractError::DoublingAssets {});
    }

    asset_infos
        .iter()
        .map(|asset_info| asset_info.validate(api))
        .collect::<Result<Vec<_>, _>>()
        .map_err(Into::into)
}

/// Stores the latest contract ownership transfer proposal
pub const OWNERSHIP_PROPOSAL: Item<OwnershipProposal> = Item::new("ownership_proposal");

/// Stores pairs to migrate
pub const PAIRS_TO_MIGRATE: Item<Vec<Addr>> = Item::new("pairs_to_migrate");

#[cfg(test)]
mod tests {
    use cosmwasm_std::testing::MockApi;
    use wyndex::asset::{native_asset_info, token_asset_info};

    use super::*;

    fn get_test_case() -> Vec<[AssetInfoValidated; 2]> {
        let api = MockApi::default();
        vec![
            [
                native_asset_info("uluna").validate(&api).unwrap(),
                native_asset_info("uusd").validate(&api).unwrap(),
            ],
            [
                native_asset_info("uluna").validate(&api).unwrap(),
                token_asset_info("astro_token_addr").validate(&api).unwrap(),
            ],
            [
                token_asset_info("random_token_addr")
                    .validate(&api)
                    .unwrap(),
                token_asset_info("astro_token_addr").validate(&api).unwrap(),
            ],
        ]
    }

    #[test]
    fn test_legacy_pair_key() {
        fn legacy_pair_key(asset_infos: &[AssetInfoValidated; 2]) -> Vec<u8> {
            let mut asset_infos = asset_infos.to_vec();
            asset_infos.sort_by(|a, b| a.as_bytes().cmp(b.as_bytes()));

            [asset_infos[0].as_bytes(), asset_infos[1].as_bytes()].concat()
        }

        for asset_infos in get_test_case() {
            assert_eq!(legacy_pair_key(&asset_infos), pair_key(&asset_infos));
        }
    }

    #[test]
    fn test_legacy_start_after() {
        fn legacy_calc_range_start(
            start_after: Option<[AssetInfoValidated; 2]>,
        ) -> Option<Vec<u8>> {
            start_after.map(|asset_infos| {
                let mut asset_infos = asset_infos.to_vec();
                asset_infos.sort_by(|a, b| a.as_bytes().cmp(b.as_bytes()));

                let mut v = [asset_infos[0].as_bytes(), asset_infos[1].as_bytes()]
                    .concat()
                    .as_slice()
                    .to_vec();
                v.push(1);
                v
            })
        }

        for asset_infos in get_test_case() {
            assert_eq!(
                legacy_calc_range_start(Some(asset_infos.clone())),
                calc_range_start(Some(asset_infos.to_vec()))
            );
        }
    }
}
