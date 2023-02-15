//! # AnsHost Entry
//! An entry (value) in the ans_host key-value store.

use crate::AbstractSdkResult;
use cosmwasm_std::{Addr, QuerierWrapper};
use cw_asset::{Asset, AssetInfo};
use os::objects::{
    ans_host::AnsHost, pool_metadata::ResolvedPoolMetadata, AnsAsset, AssetEntry, ChannelEntry,
    ContractEntry, DexAssetPairing, LpToken, PoolMetadata, PoolReference, UniquePoolId,
};

/// Resolve an [`AbstractNameService`](crate::features::AbstractNameService) entry into its value.
pub trait Resolve {
    type Output;
    fn resolve(
        &self,
        querier: &QuerierWrapper,
        ans_host: &AnsHost,
    ) -> AbstractSdkResult<Self::Output>;
}

impl Resolve for AssetEntry {
    type Output = AssetInfo;
    fn resolve(
        &self,
        querier: &QuerierWrapper,
        ans_host: &AnsHost,
    ) -> AbstractSdkResult<Self::Output> {
        ans_host.query_asset(querier, self).map_err(Into::into)
    }
}

/// TODO: this should be moved into a more appropriate package (with the LP token)
impl Resolve for LpToken {
    type Output = AssetInfo;

    fn resolve(
        &self,
        querier: &QuerierWrapper,
        ans_host: &AnsHost,
    ) -> AbstractSdkResult<Self::Output> {
        ans_host
            .query_asset(querier, &self.to_owned().into())
            .map_err(Into::into)
    }
}

impl Resolve for ContractEntry {
    type Output = Addr;
    fn resolve(
        &self,
        querier: &QuerierWrapper,
        ans_host: &AnsHost,
    ) -> AbstractSdkResult<Self::Output> {
        ans_host.query_contract(querier, self).map_err(Into::into)
    }
}

impl Resolve for ChannelEntry {
    type Output = String;
    fn resolve(
        &self,
        querier: &QuerierWrapper,
        ans_host: &AnsHost,
    ) -> AbstractSdkResult<Self::Output> {
        ans_host.query_channel(querier, self).map_err(Into::into)
    }
}

impl Resolve for DexAssetPairing {
    type Output = Vec<PoolReference>;
    fn resolve(
        &self,
        querier: &QuerierWrapper,
        ans_host: &AnsHost,
    ) -> AbstractSdkResult<Self::Output> {
        ans_host
            .query_asset_pairing(querier, self)
            .map_err(Into::into)
    }
}

impl Resolve for UniquePoolId {
    type Output = PoolMetadata;
    fn resolve(
        &self,
        querier: &QuerierWrapper,
        ans_host: &AnsHost,
    ) -> AbstractSdkResult<Self::Output> {
        ans_host
            .query_pool_metadata(querier, self)
            .map_err(Into::into)
    }
}

impl Resolve for AnsAsset {
    type Output = Asset;

    fn resolve(
        &self,
        querier: &QuerierWrapper,
        ans_host: &AnsHost,
    ) -> AbstractSdkResult<Self::Output> {
        Ok(Asset::new(
            ans_host.query_asset(querier, &self.name)?,
            self.amount,
        ))
    }
}

impl Resolve for AssetInfo {
    type Output = AssetEntry;

    fn resolve(
        &self,
        querier: &QuerierWrapper,
        ans_host: &AnsHost,
    ) -> AbstractSdkResult<Self::Output> {
        ans_host
            .query_asset_reverse(querier, self)
            .map_err(Into::into)
    }
}

impl Resolve for Asset {
    type Output = AnsAsset;

    fn resolve(
        &self,
        querier: &QuerierWrapper,
        ans_host: &AnsHost,
    ) -> AbstractSdkResult<Self::Output> {
        Ok(AnsAsset {
            name: self.info.resolve(querier, ans_host)?,
            amount: self.amount,
        })
    }
}

impl Resolve for PoolMetadata {
    type Output = ResolvedPoolMetadata;

    fn resolve(
        &self,
        querier: &QuerierWrapper,
        ans_host: &AnsHost,
    ) -> AbstractSdkResult<Self::Output> {
        Ok(ResolvedPoolMetadata {
            assets: self.assets.resolve(querier, ans_host)?,
            dex: self.dex.clone(),
            pool_type: self.pool_type.clone(),
        })
    }
}

impl<T> Resolve for Vec<T>
where
    T: Resolve,
{
    type Output = Vec<T::Output>;

    fn resolve(
        &self,
        querier: &QuerierWrapper,
        ans_host: &AnsHost,
    ) -> AbstractSdkResult<Self::Output> {
        self.iter()
            .map(|entry| entry.resolve(querier, ans_host))
            .collect()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use cosmwasm_std::Binary;

    use abstract_os::ans_host::state::ASSET_ADDRESSES;
    use abstract_testing::{wrap_querier, MockDeps, MockQuerierBuilder, TEST_ANS_HOST};
    use cosmwasm_std::{
        testing::{mock_dependencies, MockQuerier},
        Empty,
    };
    use cw_storage_plus::{Map, PrimaryKey};
    use serde::{de::DeserializeOwned, Serialize};
    use speculoos::prelude::*;

    use std::fmt::Debug;

    fn mock_map_key<'a, K, V>(map: Map<'a, K, V>, key: K) -> String
    where
        V: Serialize + DeserializeOwned,
        K: PrimaryKey<'a>,
    {
        String::from_utf8(map.key(key).deref().to_vec()).unwrap()
    }

    use std::ops::Deref;

    fn assert_not_found<T: Debug>(res: AbstractSdkResult<T>) {
        assert_that!(res)
            .is_err()
            .matches(|e| e.to_string().contains("not found"));
    }

    fn default_test_querier() -> MockQuerier {
        MockQuerierBuilder::default()
            .with_fallback_raw_handler(|contract, _| match contract {
                TEST_ANS_HOST => Ok(Binary::default()),
                _ => Err("unexpected contract".into()),
            })
            .build()
    }

    /// Querier builder with the ans host contract known.
    fn mock_deps_with_default_querier() -> MockDeps {
        let mut deps = mock_dependencies();
        deps.querier = default_test_querier();
        deps
    }

    fn mock_ans_host() -> AnsHost {
        AnsHost::new(Addr::unchecked(TEST_ANS_HOST))
    }

    pub fn test_resolve<R: Resolve>(
        querier: &MockQuerier<Empty>,
        entry: &R,
    ) -> AbstractSdkResult<R::Output> {
        entry.resolve(&wrap_querier(querier), &mock_ans_host())
    }

    fn test_dne<R: Resolve>(nonexistent: &R)
    where
        <R as Resolve>::Output: Debug,
    {
        let res = test_resolve(&default_test_querier(), nonexistent);

        assert_not_found(res);
    }

    mod asset_entry {
        use super::*;

        #[test]
        fn exists() {
            let expected_addr = Addr::unchecked("result");
            let test_asset_entry = AssetEntry::new("aoeu");
            let expected_value = AssetInfo::cw20(expected_addr.clone());
            let querier = MockQuerierBuilder::default()
                .with_contract_map_entry(
                    TEST_ANS_HOST,
                    ASSET_ADDRESSES,
                    (&test_asset_entry, &expected_value),
                )
                .build();

            let _ans_host = mock_ans_host();

            let res = test_resolve(&querier, &test_asset_entry);
            assert_that!(res).is_ok().is_equal_to(expected_value);

            let ans_asset_res = test_resolve(&querier, &AnsAsset::new("aoeu", 52256u128));
            assert_that!(ans_asset_res)
                .is_ok()
                .is_equal_to(Asset::cw20(expected_addr, 52256u128));
        }

        #[test]
        fn does_not_exist() {
            let _deps = mock_deps_with_default_querier();

            let not_exist_asset = AssetEntry::new("aoeu");

            test_dne(&not_exist_asset);
        }

        #[test]
        fn array() {
            let expected_addr = Addr::unchecked("result");
            let _expected_value = AssetInfo::cw20(expected_addr);
            let expected_entries = vec![
                (
                    AssetEntry::new("aoeu"),
                    AssetInfo::cw20(Addr::unchecked("aoeu")),
                ),
                (
                    AssetEntry::new("snth"),
                    AssetInfo::cw20(Addr::unchecked("snth")),
                ),
            ];
            let querier = MockQuerierBuilder::default()
                .with_contract_map_entries(
                    TEST_ANS_HOST,
                    ASSET_ADDRESSES,
                    expected_entries.iter().map(|(k, v)| (k, v)).collect(),
                )
                .build();

            let _ans_host = mock_ans_host();

            let (keys, values): (Vec<_>, Vec<_>) = expected_entries.into_iter().unzip();

            let res = keys.resolve(&wrap_querier(&querier), &mock_ans_host());

            assert_that!(res).is_ok().is_equal_to(values);
        }
    }

    mod lp_token {
        use super::*;

        #[test]
        fn exists() {
            let lp_token_address = Addr::unchecked("result");
            let assets = vec!["atom", "juno"];

            let test_lp_token = LpToken::new("junoswap", assets);
            let expected_value = AssetInfo::cw20(lp_token_address);
            let querier = MockQuerierBuilder::default()
                .with_contract_map_entry(
                    TEST_ANS_HOST,
                    ASSET_ADDRESSES,
                    (&test_lp_token.clone().into(), &expected_value),
                )
                .build();

            let _ans_host = mock_ans_host();

            let res = test_resolve(&querier, &test_lp_token);
            assert_that!(res).is_ok().is_equal_to(expected_value);
        }

        #[test]
        fn does_not_exist() {
            let _deps = mock_deps_with_default_querier();

            let not_exist_lp_token = LpToken::new("terraswap", vec!["rest", "peacefully"]);

            test_dne(&not_exist_lp_token);
        }
    }

    use abstract_testing::prelude::*;

    mod pool_metadata {
        use super::*;

        use os::objects::PoolType;

        #[test]
        fn exists() {
            let assets = vec!["atom", "juno"];

            let atom_addr = AssetInfo::cw20(Addr::unchecked("atom_address"));
            let juno_addr = AssetInfo::cw20(Addr::unchecked("juno_address"));
            let resolved_assets = vec![
                (AssetEntry::new("atom"), &atom_addr),
                (AssetEntry::new("juno"), &juno_addr),
            ];

            let dex = "junoswap";
            let pool_type = PoolType::ConstantProduct;
            let test_pool_metadata = PoolMetadata::new(dex.clone(), pool_type.clone(), assets);
            let querier = AbstractMockQuerierBuilder::default()
                .assets(
                    resolved_assets
                        .iter()
                        .map(|(k, v)| (k, v.clone()))
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

            let _ans_host = mock_ans_host();

            let res = test_resolve(&querier, &test_pool_metadata);
            assert_that!(res).is_ok().is_equal_to(expected_value);
        }

        #[test]
        fn does_not_exist() {
            let _deps = mock_deps_with_default_querier();

            let not_exist_md = PoolMetadata::new(
                "junoswap",
                PoolType::ConstantProduct,
                vec![AssetEntry::new("juno")],
            );

            test_dne(&not_exist_md);
        }
    }

    mod pools {
        use super::*;
        use abstract_os::ans_host::state::{ASSET_PAIRINGS, POOL_METADATA};
        use os::objects::{PoolAddress, PoolType};

        #[test]
        fn exists() {
            let _pool_address = Addr::unchecked("result");
            let assets = vec!["atom", "juno"];
            let dex = "boogerswap";
            let pairing = DexAssetPairing::new(
                AssetEntry::new(assets[0].clone()),
                AssetEntry::new(assets[1].clone()),
                dex.clone(),
            );

            let unique_pool_id: UniquePoolId = 1u64.into();
            let pool_address: PoolAddress = Addr::unchecked("pool_address").into();
            let pool_reference = PoolReference::new(unique_pool_id, pool_address);
            let pool_metadata =
                PoolMetadata::new(dex.clone(), PoolType::ConstantProduct, assets.clone());

            let querier = MockQuerierBuilder::default()
                .with_contract_map_entry(
                    TEST_ANS_HOST,
                    ASSET_PAIRINGS,
                    (&pairing, &vec![pool_reference]),
                )
                .with_contract_map_entry(
                    TEST_ANS_HOST,
                    POOL_METADATA,
                    (unique_pool_id, &pool_metadata),
                )
                .build();

            let _ans_host = mock_ans_host();

            let unique_pool_id_res = test_resolve(&querier, &unique_pool_id);
            assert_that!(unique_pool_id_res)
                .is_ok()
                .is_equal_to(pool_metadata);
        }

        #[test]
        fn does_not_exist() {
            let _deps = mock_deps_with_default_querier();

            let not_exist_pool = UniquePoolId::new(1u64);

            test_dne(&not_exist_pool);
        }
    }

    mod contract_entry {
        use super::*;
        use os::ans_host::state::CONTRACT_ADDRESSES;

        #[test]
        fn exists() {
            let test_contract_entry = ContractEntry {
                protocol: "protocol".to_string(),
                contract: "contract".to_string(),
            };

            let expected_value = Addr::unchecked("address");
            let querier = MockQuerierBuilder::default()
                .with_contract_map_entry(
                    TEST_ANS_HOST,
                    CONTRACT_ADDRESSES,
                    (&test_contract_entry, &expected_value),
                )
                .build();

            let res = test_resolve(&querier, &test_contract_entry);

            assert_that!(res).is_ok().is_equal_to(expected_value);
        }

        #[test]
        fn does_not_exist() {
            let not_exist_contract = ContractEntry {
                protocol: "protocol".to_string(),
                contract: "contract".to_string(),
            };

            test_dne(&not_exist_contract);
        }

        #[test]
        fn array() {
            let expected_addr = Addr::unchecked("result");
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
                    TEST_ANS_HOST,
                    CONTRACT_ADDRESSES,
                    expected_entries.iter().map(|(k, v)| (k, v)).collect(),
                )
                .build();

            let (keys, values): (Vec<_>, Vec<_>) = expected_entries.into_iter().unzip();

            let res = keys.resolve(&wrap_querier(&querier), &mock_ans_host());

            assert_that!(res).is_ok().is_equal_to(values);
        }
    }

    mod channel_entry {
        use super::*;
        use os::ans_host::state::CHANNELS;

        #[test]
        fn exists() {
            let test_channel_entry = ChannelEntry {
                protocol: "protocol".to_string(),
                connected_chain: "abstract".to_string(),
            };

            let expected_value = "channel-id".to_string();
            let querier = MockQuerierBuilder::default()
                .with_contract_map_entry(
                    TEST_ANS_HOST,
                    CHANNELS,
                    (&test_channel_entry, &expected_value),
                )
                .build();

            let res = test_resolve(&querier, &test_channel_entry);

            assert_that!(res).is_ok().is_equal_to(expected_value);
        }

        #[test]
        fn does_not_exist() {
            let not_exist_channel = ChannelEntry {
                protocol: "protocol".to_string(),
                connected_chain: "chain".to_string(),
            };

            test_dne(&not_exist_channel);
        }
    }

    mod asset_info_and_asset {
        use super::*;
        use os::ans_host::state::REV_ASSET_ADDRESSES;

        #[test]
        fn exists() {
            let expected_address = Addr::unchecked("address");
            let test_asset_info = AssetInfo::cw20(expected_address.clone());

            let expected_value = AssetEntry::new("chinachinachina");
            let querier = MockQuerierBuilder::default()
                .with_contract_map_entry(
                    TEST_ANS_HOST,
                    REV_ASSET_ADDRESSES,
                    (&test_asset_info, &expected_value),
                )
                .build();

            let res = test_resolve(&querier, &test_asset_info);
            assert_that!(res).is_ok().is_equal_to(expected_value);

            let asset_res = test_resolve(&querier, &Asset::cw20(expected_address, 12345u128));
            assert_that!(asset_res)
                .is_ok()
                .is_equal_to(AnsAsset::new("chinachinachina", 12345u128));
        }

        #[test]
        fn does_not_exist() {
            let not_exist_asset_info = AssetInfo::cw20(Addr::unchecked("address"));

            test_dne(&not_exist_asset_info);
        }

        #[test]
        fn array() {
            let expected_entries = vec![
                (
                    AssetInfo::cw20(Addr::unchecked("boop")),
                    AssetEntry::new("beepboop"),
                ),
                (
                    AssetInfo::cw20(Addr::unchecked("iloveabstract")),
                    AssetEntry::new("robinrocks!"),
                ),
            ];
            let querier = MockQuerierBuilder::default()
                .with_contract_map_entries(
                    TEST_ANS_HOST,
                    REV_ASSET_ADDRESSES,
                    expected_entries.iter().map(|(k, v)| (k, v)).collect(),
                )
                .build();

            let (keys, values): (Vec<_>, Vec<_>) = expected_entries.into_iter().unzip();

            let res = keys.resolve(&wrap_querier(&querier), &mock_ans_host());

            assert_that!(res).is_ok().is_equal_to(values);
        }
    }
}
