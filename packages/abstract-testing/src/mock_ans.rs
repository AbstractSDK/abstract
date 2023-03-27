use abstract_core::{
    ans_host::{
        state::{
            ASSET_ADDRESSES, ASSET_PAIRINGS, CHANNELS, CONTRACT_ADDRESSES, POOL_METADATA,
            REGISTERED_DEXES,
        },
        AssetPair,
    },
    objects::{
        pool_id::UncheckedPoolAddress, AssetEntry, ChannelEntry, ContractEntry, DexAssetPairing,
        PoolMetadata, PoolReference, PoolType, UniquePoolId,
    },
};
use cosmwasm_std::{
    testing::{MockApi, MockQuerier},
    Addr,
};
use cw_asset::AssetInfo;

use crate::{addresses::*, MockQuerierBuilder};

/// mirror ANS state
/// ```rust,ignore
/// pub const ASSET_ADDRESSES: Map<&AssetEntry, AssetInfo> = Map::new("assets");
/// pub const REV_ASSET_ADDRESSES: Map<&AssetInfo, AssetEntry> = Map::new("rev_assets");
/// pub const CONTRACT_ADDRESSES: Map<&ContractEntry, Addr> = Map::new("contracts");
/// pub const CHANNELS: Map<&ChannelEntry, String> = Map::new("channels");
/// pub const REGISTERED_DEXES: Item<Vec<DexName>> = Item::new("registered_dexes");
/// // Stores the asset pairing entries to their pool ids
/// // (asset1, asset2, dex_name) -> {id: uniqueId, pool_id: poolId}
/// pub const ASSET_PAIRINGS: Map<&DexAssetPairing, Vec<PoolReference>> = Map::new("pool_ids");
/// pub const POOL_METADATA: Map<UniquePoolId, PoolMetadata> = Map::new("pools");
/// ```
#[derive(Default)]
pub struct MockAnsHost {
    pub contracts: Vec<(ContractEntry, Addr)>,
    pub assets: Vec<(AssetEntry, AssetInfo)>,
    pub channels: Vec<(ChannelEntry, String)>,
    pub pools: Vec<(UncheckedPoolAddress, PoolMetadata)>,
}

impl MockAnsHost {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn to_querier(self) -> MockQuerier {
        self.insert_into(MockQuerierBuilder::default()).build()
    }

    pub fn insert_into(self, querier_builder: MockQuerierBuilder) -> MockQuerierBuilder {
        let mut querier_builder = querier_builder
            .with_contract_map_entries(
                TEST_ANS_HOST,
                ASSET_ADDRESSES,
                self.assets.iter().map(|(a, b)| (a, b.clone())).collect(),
            )
            .with_contract_map_entries(
                TEST_ANS_HOST,
                CONTRACT_ADDRESSES,
                self.contracts.iter().map(|(a, b)| (a, b.clone())).collect(),
            )
            .with_contract_map_entries(
                TEST_ANS_HOST,
                CHANNELS,
                self.channels.iter().map(|(a, b)| (a, b.clone())).collect(),
            );

        let mut unique_id = UniquePoolId::new(0);
        let mut dexes = vec![];
        for (pool_addr, pool_meta) in self.pools {
            let dex = pool_meta.dex.clone();
            if !dexes.contains(&dex) {
                dexes.push(dex);
            }
            let pool_addr = pool_addr.check(&MockApi::default()).unwrap();
            querier_builder = querier_builder.with_contract_map_entries(
                TEST_ANS_HOST,
                POOL_METADATA,
                vec![(unique_id, pool_meta.clone())],
            );
            // add pairs for this pool
            for (i, asset_x) in pool_meta.assets.iter().enumerate() {
                for (j, asset_y) in pool_meta.assets.iter().enumerate() {
                    // Skip self-pairings
                    if i == j || asset_x == asset_y {
                        continue;
                    }
                    let pair: AssetPair = (asset_x.clone(), asset_y.clone());
                    let pair: DexAssetPairing =
                        DexAssetPairing::new(pair.0.clone(), pair.1.clone(), &pool_meta.dex);
                    querier_builder = querier_builder.with_contract_map_entries(
                        TEST_ANS_HOST,
                        ASSET_PAIRINGS,
                        vec![(
                            &pair,
                            vec![PoolReference {
                                unique_id,
                                pool_address: pool_addr.clone(),
                            }],
                        )],
                    );
                }
            }

            // .iter().for_each(|pair: Vec<AssetEntry>| {
            //     if pair[0] == pair[1] {
            //         return;
            //     }
            //     let pair: DexAssetPairing = DexAssetPairing::new(pair[0].clone(), pair[1].clone(), &pool_meta.dex);
            //     querier_builder.with_contract_map_entries(TEST_ANS_HOST, ASSET_PAIRINGS, vec![(&pair, vec![PoolReference{ unique_id, pool_address: pool_addr.clone() }])]);
            // });

            unique_id.increment();
        }
        querier_builder.with_contract_item(TEST_ANS_HOST, REGISTERED_DEXES, &dexes)
    }

    pub fn with_defaults(mut self) -> Self {
        self.assets.append(&mut vec![
            (AssetEntry::from(EUR), AssetInfo::native(EUR)),
            (AssetEntry::from(USD), AssetInfo::native(USD)),
            (
                AssetEntry::from(TTOKEN),
                AssetInfo::Cw20(Addr::unchecked(TTOKEN)),
            ),
            (
                AssetEntry::from(EUR_USD_LP),
                AssetInfo::Cw20(Addr::unchecked(EUR_USD_LP)),
            ),
            (
                AssetEntry::from(TTOKEN_EUR_LP),
                AssetInfo::Cw20(Addr::unchecked(TTOKEN_EUR_LP)),
            ),
        ]);
        self.pools.append(&mut vec![
            (
                UncheckedPoolAddress::contract(EUR_USD_PAIR),
                PoolMetadata::new(
                    TEST_DEX,
                    PoolType::ConstantProduct,
                    vec![AssetEntry::from(EUR), AssetEntry::from(USD)],
                ),
            ),
            (
                UncheckedPoolAddress::contract(TTOKEN_EUR_PAIR),
                PoolMetadata::new(
                    TEST_DEX,
                    PoolType::ConstantProduct,
                    vec![AssetEntry::from(TTOKEN), AssetEntry::from(EUR)],
                ),
            ),
        ]);
        self
    }
}
