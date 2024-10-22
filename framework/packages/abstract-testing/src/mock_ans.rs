use abstract_std::{
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

use crate::{addresses::*, ans::*, MockQuerierBuilder};

/// mirror ANS state
/// ```rust,ignore
/// use abstract_std::ans_host::state::{
///     ASSET_ADDRESSES, ASSET_PAIRINGS, CHANNELS, CONTRACT_ADDRESSES, POOL_METADATA,
///     REGISTERED_DEXES,
/// };
/// ```
pub struct MockAnsHost {
    pub contracts: Vec<(ContractEntry, Addr)>,
    pub assets: Vec<(AssetEntry, AssetInfo)>,
    pub channels: Vec<(ChannelEntry, String)>,
    pub pools: Vec<(UncheckedPoolAddress, PoolMetadata)>,
    pub mock_api: MockApi,
}

impl MockAnsHost {
    pub fn new(mock_api: MockApi) -> Self {
        Self {
            contracts: vec![],
            assets: vec![],
            channels: vec![],
            pools: vec![],
            mock_api,
        }
    }

    pub fn to_querier(self) -> MockQuerier {
        self.insert_into(MockQuerierBuilder::default()).build()
    }

    pub fn insert_into(self, querier_builder: MockQuerierBuilder) -> MockQuerierBuilder {
        let abstract_addrs = AbstractMockAddrs::new(self.mock_api);
        let mut querier_builder = querier_builder
            .with_contract_map_entries(
                &abstract_addrs.ans_host,
                ASSET_ADDRESSES,
                self.assets.iter().map(|(a, b)| (a, b.clone())).collect(),
            )
            .with_contract_map_entries(
                &abstract_addrs.ans_host,
                CONTRACT_ADDRESSES,
                self.contracts.iter().map(|(a, b)| (a, b.clone())).collect(),
            )
            .with_contract_map_entries(
                &abstract_addrs.ans_host,
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
                &abstract_addrs.ans_host,
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
                        &abstract_addrs.ans_host,
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
        querier_builder.with_contract_item(&abstract_addrs.ans_host, REGISTERED_DEXES, &dexes)
    }

    pub fn with_defaults(mut self) -> Self {
        self.assets.extend([
            (AssetEntry::from(EUR), AssetInfo::native(EUR)),
            (AssetEntry::from(USD), AssetInfo::native(USD)),
            (
                AssetEntry::from(TTOKEN),
                AssetInfo::Cw20(self.mock_api.addr_make(TTOKEN)),
            ),
            (
                AssetEntry::from(EUR_USD_LP),
                AssetInfo::Cw20(self.mock_api.addr_make(EUR_USD_LP)),
            ),
            (
                AssetEntry::from(TTOKEN_EUR_LP),
                AssetInfo::Cw20(self.mock_api.addr_make(TTOKEN_EUR_LP)),
            ),
        ]);
        self.pools.extend([
            (
                UncheckedPoolAddress::contract(self.mock_api.addr_make(EUR_USD_PAIR).into_string()),
                PoolMetadata::new(
                    TEST_DEX,
                    PoolType::ConstantProduct,
                    vec![AssetEntry::from(EUR), AssetEntry::from(USD)],
                ),
            ),
            (
                UncheckedPoolAddress::contract(
                    self.mock_api.addr_make(TTOKEN_EUR_PAIR).into_string(),
                ),
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
