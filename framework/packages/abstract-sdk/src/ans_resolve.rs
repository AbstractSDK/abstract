//! # AnsHost Entry
//! An entry (value) in the ans_host key-value store.

use abstract_std::objects::{ans_host::AnsHostResult, AnsEntryConvertor};
use cosmwasm_std::{Addr, QuerierWrapper};
use cw_asset::{Asset, AssetInfo};

use crate::std::objects::{
    ans_host::AnsHost, pool_metadata::ResolvedPoolMetadata, AnsAsset, AssetEntry, ChannelEntry,
    ContractEntry, DexAssetPairing, LpToken, PoolMetadata, PoolReference, UniquePoolId,
};

/// Resolve an [`AbstractNameService`](crate::features::AbstractNameService) entry into its value.
pub trait Resolve {
    /// Result of resolving an entry.
    type Output;
    /// Resolve an entry into its value.
    fn resolve(&self, querier: &QuerierWrapper, ans_host: &AnsHost) -> AnsHostResult<Self::Output>;
    /// Check if the entry is registered in the ANS.
    fn is_registered(&self, querier: &QuerierWrapper, ans_host: &AnsHost) -> bool {
        self.resolve(querier, ans_host).is_ok()
    }
    /// Assert that a given entry is registered in the ANS.
    fn assert_registered(&self, querier: &QuerierWrapper, ans_host: &AnsHost) -> AnsHostResult<()> {
        self.resolve(querier, ans_host).map(|_| ())
    }
}

impl Resolve for AssetEntry {
    type Output = AssetInfo;
    fn resolve(&self, querier: &QuerierWrapper, ans_host: &AnsHost) -> AnsHostResult<Self::Output> {
        ans_host.query_asset(querier, self)
    }
}

/// TODO: this should be moved into a more appropriate package (with the LP token)
impl Resolve for LpToken {
    type Output = AssetInfo;

    fn resolve(&self, querier: &QuerierWrapper, ans_host: &AnsHost) -> AnsHostResult<Self::Output> {
        let asset_entry = AnsEntryConvertor::new(self.clone()).asset_entry();
        ans_host.query_asset(querier, &asset_entry)
    }
}

impl Resolve for ContractEntry {
    type Output = Addr;
    fn resolve(&self, querier: &QuerierWrapper, ans_host: &AnsHost) -> AnsHostResult<Self::Output> {
        ans_host.query_contract(querier, self)
    }
}

impl Resolve for ChannelEntry {
    type Output = String;
    fn resolve(&self, querier: &QuerierWrapper, ans_host: &AnsHost) -> AnsHostResult<Self::Output> {
        ans_host.query_channel(querier, self)
    }
}

impl Resolve for DexAssetPairing {
    type Output = Vec<PoolReference>;
    fn resolve(&self, querier: &QuerierWrapper, ans_host: &AnsHost) -> AnsHostResult<Self::Output> {
        ans_host.query_asset_pairing(querier, self)
    }
}

impl Resolve for UniquePoolId {
    type Output = PoolMetadata;
    fn resolve(&self, querier: &QuerierWrapper, ans_host: &AnsHost) -> AnsHostResult<Self::Output> {
        ans_host.query_pool_metadata(querier, *self)
    }
}

impl Resolve for AnsAsset {
    type Output = Asset;

    fn resolve(&self, querier: &QuerierWrapper, ans_host: &AnsHost) -> AnsHostResult<Self::Output> {
        Ok(Asset::new(
            ans_host.query_asset(querier, &self.name)?,
            self.amount,
        ))
    }
}

impl Resolve for AssetInfo {
    type Output = AssetEntry;

    fn resolve(&self, querier: &QuerierWrapper, ans_host: &AnsHost) -> AnsHostResult<Self::Output> {
        ans_host.query_asset_reverse(querier, self)
    }
}

impl Resolve for Asset {
    type Output = AnsAsset;

    fn resolve(&self, querier: &QuerierWrapper, ans_host: &AnsHost) -> AnsHostResult<Self::Output> {
        Ok(AnsAsset {
            name: self.info.resolve(querier, ans_host)?,
            amount: self.amount,
        })
    }
}

impl Resolve for PoolMetadata {
    type Output = ResolvedPoolMetadata;

    fn resolve(&self, querier: &QuerierWrapper, ans_host: &AnsHost) -> AnsHostResult<Self::Output> {
        Ok(ResolvedPoolMetadata {
            assets: self.assets.resolve(querier, ans_host)?,
            dex: self.dex.clone(),
            pool_type: self.pool_type,
        })
    }
}

impl<T> Resolve for Vec<T>
where
    T: Resolve,
{
    type Output = Vec<T::Output>;

    fn resolve(&self, querier: &QuerierWrapper, ans_host: &AnsHost) -> AnsHostResult<Self::Output> {
        self.iter()
            .map(|entry| entry.resolve(querier, ans_host))
            .collect()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    use abstract_std::ans_host::state::ASSET_ADDRESSES;
    use abstract_testing::prelude::*;
    use cosmwasm_std::{
        testing::{mock_dependencies, MockApi},
        Binary, Empty,
    };
    use speculoos::prelude::*;
    use std::fmt::Debug;

    fn default_test_querier(ans_host: &AnsHost) -> MockQuerier {
        let ans_host_addr = ans_host.address.clone();
        MockQuerierBuilder::default()
            .with_fallback_raw_handler(move |contract, _| {
                if contract == ans_host_addr {
                    Ok(Binary::default())
                } else {
                    Err("unexpected contract".into())
                }
            })
            .build()
    }

    fn mock_ans_host(mock_api: MockApi) -> AnsHost {
        let abstract_addrs = AbstractMockAddrs::new(mock_api);
        AnsHost::new(abstract_addrs.ans_host)
    }

    /// Querier builder with the ans host contract known.
    fn mock_deps_with_default_querier() -> MockDeps {
        let mut deps = mock_dependencies();
        let ans_host = mock_ans_host(deps.api);
        deps.querier = default_test_querier(&ans_host);
        deps
    }

    pub fn test_resolve<R: Resolve>(
        ans_host: &AnsHost,
        querier: &MockQuerier<Empty>,
        entry: &R,
    ) -> AnsHostResult<R::Output> {
        entry.resolve(&wrap_querier(querier), ans_host)
    }

    fn test_dne<R: Resolve>(ans_host: &AnsHost, nonexistent: &R)
    where
        <R as Resolve>::Output: Debug,
    {
        let res = test_resolve(ans_host, &default_test_querier(ans_host), nonexistent);

        assert_that!(res)
            .is_err()
            .matches(|e| e.to_string().contains("not found"));
    }

    mod is_registered {
        use super::*;

        #[test]
        fn exists() {
            let mock_api = MockApi::default();
            let ans_host = mock_ans_host(mock_api);

            let test_asset_entry = AssetEntry::new("aoeu");
            let querier = MockQuerierBuilder::default()
                .with_contract_map_entry(
                    &ans_host.address,
                    ASSET_ADDRESSES,
                    (&test_asset_entry, AssetInfo::native("abc")),
                )
                .build();

            let is_registered =
                test_asset_entry.is_registered(&QuerierWrapper::new(&querier), &ans_host);
            assert_that!(is_registered).is_true();
        }

        #[test]
        fn does_not_exist() {
            let mock_api = MockApi::default();
            let ans_host = mock_ans_host(mock_api);

            let not_exist_asset = AssetEntry::new("aoeu");
            let querier = default_test_querier(&ans_host);
            let wrapper = wrap_querier(&querier);

            let is_registered = not_exist_asset.is_registered(&wrapper, &ans_host);
            assert_that!(is_registered).is_false();
        }
    }

    mod asset_entry {
        use super::*;

        #[test]
        fn exists() {
            let mock_api = MockApi::default();
            let ans_host = mock_ans_host(mock_api);

            let expected_addr = Addr::unchecked("result");
            let test_asset_entry = AssetEntry::new("aoeu");
            let expected_value = AssetInfo::cw20(expected_addr.clone());
            let querier = MockQuerierBuilder::default()
                .with_contract_map_entry(
                    &ans_host.address,
                    ASSET_ADDRESSES,
                    (&test_asset_entry, expected_value.clone()),
                )
                .build();

            let res = test_resolve(&ans_host, &querier, &test_asset_entry);
            assert_that!(res).is_ok().is_equal_to(expected_value);

            let ans_asset_res =
                test_resolve(&ans_host, &querier, &AnsAsset::new("aoeu", 52256u128));
            assert_that!(ans_asset_res)
                .is_ok()
                .is_equal_to(Asset::cw20(expected_addr, 52256u128));
        }

        #[test]
        fn does_not_exist() {
            let deps = mock_deps_with_default_querier();
            let ans_host = mock_ans_host(deps.api);

            let not_exist_asset = AssetEntry::new("aoeu");

            test_dne(&ans_host, &not_exist_asset);
        }

        #[test]
        fn array() {
            let mock_api = MockApi::default();
            let ans_host = mock_ans_host(mock_api);

            let expected_entries = vec![
                (
                    AssetEntry::new("aoeu"),
                    AssetInfo::cw20(mock_api.addr_make("aoeu")),
                ),
                (
                    AssetEntry::new("snth"),
                    AssetInfo::cw20(mock_api.addr_make("snth")),
                ),
            ];
            let querier = MockQuerierBuilder::default()
                .with_contract_map_entries(
                    &ans_host.address,
                    ASSET_ADDRESSES,
                    expected_entries
                        .iter()
                        .map(|(k, v)| (k, v.clone()))
                        .collect(),
                )
                .build();

            let (keys, values): (Vec<_>, Vec<_>) = expected_entries.into_iter().unzip();

            let res = keys.resolve(&wrap_querier(&querier), &ans_host);

            assert_that!(res).is_ok().is_equal_to(values);
        }
    }

    mod lp_token {
        use super::*;

        #[test]
        fn exists() {
            let mock_api = MockApi::default();
            let ans_host = mock_ans_host(mock_api);

            let lp_token_address = mock_api.addr_make("result");
            let assets = vec!["atom", "juno"];

            let test_lp_token = LpToken::new("junoswap", assets);
            let asset_entry = AnsEntryConvertor::new(test_lp_token.clone()).asset_entry();
            let expected_value = AssetInfo::cw20(lp_token_address);
            let querier = MockQuerierBuilder::default()
                .with_contract_map_entry(
                    &ans_host.address,
                    ASSET_ADDRESSES,
                    (&asset_entry, expected_value.clone()),
                )
                .build();

            let res = test_resolve(&ans_host, &querier, &test_lp_token);
            assert_that!(res).is_ok().is_equal_to(expected_value);
        }

        #[test]
        fn does_not_exist() {
            let mock_api = MockApi::default();
            let ans_host = mock_ans_host(mock_api);

            let not_exist_lp_token = LpToken::new("terraswap", vec!["rest", "peacefully"]);

            test_dne(&ans_host, &not_exist_lp_token);
        }
    }

    mod pool_metadata {
        use super::*;
        use crate::std::objects::PoolType;

        #[test]
        fn exists() {
            let mock_api = MockApi::default();
            let ans_host = mock_ans_host(mock_api);

            let assets = vec!["atom", "juno"];

            let atom_addr = AssetInfo::cw20(mock_api.addr_make("atom_address"));
            let juno_addr = AssetInfo::cw20(mock_api.addr_make("juno_address"));
            let resolved_assets = vec![
                (AssetEntry::new("atom"), &atom_addr),
                (AssetEntry::new("juno"), &juno_addr),
            ];

            let dex = "junoswap";
            let pool_type = PoolType::ConstantProduct;
            let test_pool_metadata = PoolMetadata::new(dex, pool_type, assets);
            let querier = AbstractMockQuerierBuilder::new(mock_api)
                .assets(
                    resolved_assets
                        .iter()
                        .map(|(k, v)| (k, (*v).clone()))
                        .collect(),
                )
                .build();

            let expected_value = ResolvedPoolMetadata {
                dex: dex.into(),
                pool_type,
                assets: resolved_assets
                    .into_iter()
                    .map(|(_, b)| b.clone())
                    .collect(),
            };

            let res = test_resolve(&ans_host, &querier, &test_pool_metadata);
            assert_that!(res).is_ok().is_equal_to(expected_value);
        }

        #[test]
        fn does_not_exist() {
            let mock_api = MockApi::default();
            let ans_host = mock_ans_host(mock_api);

            let not_exist_md = PoolMetadata::new(
                "junoswap",
                PoolType::ConstantProduct,
                vec![AssetEntry::new("juno")],
            );

            test_dne(&ans_host, &not_exist_md);
        }
    }

    mod pools {
        use abstract_std::ans_host::state::{ASSET_PAIRINGS, POOL_METADATA};

        use super::*;
        use crate::std::objects::{PoolAddress, PoolType};

        #[test]
        fn exists() {
            let mock_api = MockApi::default();
            let ans_host = mock_ans_host(mock_api);

            let assets = vec!["atom", "juno"];
            let dex = "boogerswap";
            let pairing =
                DexAssetPairing::new(AssetEntry::new(assets[0]), AssetEntry::new(assets[1]), dex);

            let unique_pool_id: UniquePoolId = 1u64.into();
            let pool_address: PoolAddress = mock_api.addr_make("pool_address").into();
            let pool_reference = PoolReference::new(unique_pool_id, pool_address);
            let pool_metadata = PoolMetadata::new(dex, PoolType::ConstantProduct, assets.clone());

            let querier = MockQuerierBuilder::default()
                .with_contract_map_entry(
                    &ans_host.address,
                    ASSET_PAIRINGS,
                    (&pairing, vec![pool_reference]),
                )
                .with_contract_map_entry(
                    &ans_host.address,
                    POOL_METADATA,
                    (unique_pool_id, pool_metadata.clone()),
                )
                .build();

            let unique_pool_id_res = test_resolve(&ans_host, &querier, &unique_pool_id);
            assert_that!(unique_pool_id_res)
                .is_ok()
                .is_equal_to(pool_metadata);
        }

        #[test]
        fn does_not_exist() {
            let mock_api = MockApi::default();
            let ans_host = mock_ans_host(mock_api);

            let not_exist_pool = UniquePoolId::new(1u64);

            test_dne(&ans_host, &not_exist_pool);
        }
    }

    mod contract_entry {
        use super::*;
        use crate::std::ans_host::state::CONTRACT_ADDRESSES;

        #[test]
        fn exists() {
            let mock_api = MockApi::default();
            let ans_host = mock_ans_host(mock_api);

            let test_contract_entry = ContractEntry {
                protocol: "protocol".to_string(),
                contract: "contract".to_string(),
            };

            let expected_value = mock_api.addr_make("address");
            let querier = MockQuerierBuilder::default()
                .with_contract_map_entry(
                    &ans_host.address,
                    CONTRACT_ADDRESSES,
                    (&test_contract_entry, expected_value.clone()),
                )
                .build();

            let res = test_resolve(&ans_host, &querier, &test_contract_entry);

            assert_that!(res).is_ok().is_equal_to(expected_value);
        }

        #[test]
        fn does_not_exist() {
            let mock_api = MockApi::default();
            let ans_host = mock_ans_host(mock_api);

            let not_exist_contract = ContractEntry {
                protocol: "protocol".to_string(),
                contract: "contract".to_string(),
            };

            test_dne(&ans_host, &not_exist_contract);
        }

        #[test]
        fn array() {
            let mock_api = MockApi::default();
            let ans_host = mock_ans_host(mock_api);

            let expected_addr = mock_api.addr_make("result");
            let expected_entries = vec![
                (
                    ContractEntry {
                        protocol: "junoswap".to_string(),
                        contract: "something".to_string(),
                    },
                    expected_addr.clone(),
                ),
                (
                    ContractEntry {
                        protocol: "astroport".to_string(),
                        contract: "something".to_string(),
                    },
                    expected_addr,
                ),
            ];
            let querier = MockQuerierBuilder::default()
                .with_contract_map_entries(
                    &ans_host.address,
                    CONTRACT_ADDRESSES,
                    expected_entries
                        .iter()
                        .map(|(k, v)| (k, v.clone()))
                        .collect(),
                )
                .build();

            let (keys, values): (Vec<_>, Vec<_>) = expected_entries.into_iter().unzip();

            let res = keys.resolve(&wrap_querier(&querier), &ans_host);

            assert_that!(res).is_ok().is_equal_to(values);
        }
    }

    mod channel_entry {
        use std::str::FromStr;

        use abstract_std::objects::TruncatedChainId;

        use super::*;
        use crate::std::ans_host::state::CHANNELS;

        #[test]
        fn exists() {
            let mock_api = MockApi::default();
            let ans_host = mock_ans_host(mock_api);

            let test_channel_entry = ChannelEntry {
                protocol: "protocol".to_string(),
                connected_chain: TruncatedChainId::from_str("abstract").unwrap(),
            };

            let expected_value = "channel-id".to_string();
            let querier = MockQuerierBuilder::default()
                .with_contract_map_entry(
                    &ans_host.address,
                    CHANNELS,
                    (&test_channel_entry, expected_value.clone()),
                )
                .build();

            let res = test_resolve(&ans_host, &querier, &test_channel_entry);

            assert_that!(res).is_ok().is_equal_to(expected_value);
        }

        #[test]
        fn does_not_exist() {
            let mock_api = MockApi::default();
            let ans_host = mock_ans_host(mock_api);

            let not_exist_channel = ChannelEntry {
                protocol: "protocol".to_string(),
                connected_chain: TruncatedChainId::from_str("chain").unwrap(),
            };

            test_dne(&ans_host, &not_exist_channel);
        }
    }

    mod asset_info_and_asset {
        use super::*;
        use crate::std::ans_host::state::REV_ASSET_ADDRESSES;

        #[test]
        fn exists() {
            let mock_api = MockApi::default();
            let ans_host = mock_ans_host(mock_api);

            let expected_address = mock_api.addr_make("address");
            let test_asset_info = AssetInfo::cw20(expected_address.clone());

            let expected_value = AssetEntry::new("chinachinachina");
            let querier = MockQuerierBuilder::default()
                .with_contract_map_entry(
                    &ans_host.address,
                    REV_ASSET_ADDRESSES,
                    (&test_asset_info, expected_value.clone()),
                )
                .build();

            let res = test_resolve(&ans_host, &querier, &test_asset_info);
            assert_that!(res).is_ok().is_equal_to(expected_value);

            let asset_res = test_resolve(
                &ans_host,
                &querier,
                &Asset::cw20(expected_address, 12345u128),
            );
            assert_that!(asset_res)
                .is_ok()
                .is_equal_to(AnsAsset::new("chinachinachina", 12345u128));
        }

        #[test]
        fn does_not_exist() {
            let mock_api = MockApi::default();
            let ans_host = mock_ans_host(mock_api);

            let not_exist_asset_info = AssetInfo::cw20(mock_api.addr_make("address"));

            test_dne(&ans_host, &not_exist_asset_info);
        }

        #[test]
        fn array() {
            let mock_api = MockApi::default();
            let ans_host = mock_ans_host(mock_api);

            let expected_entries = vec![
                (
                    AssetInfo::cw20(mock_api.addr_make("boop")),
                    AssetEntry::new("beepboop"),
                ),
                (
                    AssetInfo::cw20(mock_api.addr_make("iloveabstract")),
                    AssetEntry::new("robinrocks!"),
                ),
            ];
            let querier = MockQuerierBuilder::default()
                .with_contract_map_entries(
                    &ans_host.address,
                    REV_ASSET_ADDRESSES,
                    expected_entries
                        .iter()
                        .map(|(k, v)| (k, v.clone()))
                        .collect(),
                )
                .build();

            let (keys, values): (Vec<_>, Vec<_>) = expected_entries.into_iter().unzip();

            let res = keys.resolve(&wrap_querier(&querier), &ans_host);

            assert_that!(res).is_ok().is_equal_to(values);
        }
    }
}
