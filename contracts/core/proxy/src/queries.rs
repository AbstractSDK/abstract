use crate::contract::ProxyResult;

use abstract_os::objects::oracle::{AccountValue, Oracle};
use abstract_os::proxy::{
    AssetsConfigResponse, BaseAssetResponse, HoldingAmountResponse, OracleAsset,
};
use abstract_sdk::os::objects::AssetEntry;
use abstract_sdk::os::proxy::state::{ANS_HOST, STATE};
use abstract_sdk::os::proxy::{AssetsInfoResponse, ConfigResponse};
use abstract_sdk::Resolve;
use cosmwasm_std::{Addr, Deps, Env, StdResult, Uint128};
use cw_asset::{Asset, AssetInfo};

/// get the assets pricing information
pub fn query_oracle_asset_info(
    deps: Deps,
    last_asset: Option<AssetInfo>,
    limit: Option<u8>,
) -> ProxyResult<AssetsInfoResponse> {
    let oracle = Oracle::new();
    let assets = oracle.paged_asset_info(deps, last_asset, limit)?;
    Ok(AssetsInfoResponse {
        assets: assets
            .into_iter()
            .map(|(a, (p, c))| {
                (
                    a,
                    OracleAsset {
                        complexity: c,
                        price_source: p,
                    },
                )
            })
            .collect(),
    })
}

/// get the human-readable asset pricing information
pub fn query_oracle_asset_config(
    deps: Deps,
    last_asset: Option<AssetEntry>,
    limit: Option<u8>,
) -> ProxyResult<AssetsConfigResponse> {
    let oracle = Oracle::new();
    let assets = oracle.paged_asset_config(deps, last_asset, limit)?;
    Ok(AssetsConfigResponse { assets })
}

/// Returns the whitelisted modules
pub fn query_config(deps: Deps) -> StdResult<ConfigResponse> {
    let state = STATE.load(deps.storage)?;
    let modules: Vec<Addr> = state.modules;
    let resp = ConfigResponse {
        modules: modules
            .iter()
            .map(|module| -> String { module.to_string() })
            .collect(),
    };
    Ok(resp)
}

/// Returns the value of the amount of the specified asset
/// @param amount: The amount of the asset to compute the value of. If None, balance of the proxy account is used.
pub fn query_token_value(
    deps: Deps,
    env: Env,
    asset_entry: AssetEntry,
    amount: Option<Uint128>,
) -> ProxyResult<Uint128> {
    let oracle = Oracle::new();
    let ans_host = ANS_HOST.load(deps.storage)?;
    let asset_info = asset_entry.resolve(&deps.querier, &ans_host)?;
    let balance = amount.unwrap_or_else(|| {
        asset_info
            .query_balance(&deps.querier, env.contract.address)
            .unwrap()
    });
    let value = oracle.asset_value(deps, Asset::new(asset_info, balance))?;
    Ok(value)
}

/// Computes the total value locked in this contract
pub fn query_total_value(deps: Deps, env: Env) -> ProxyResult<AccountValue> {
    let mut oracle = Oracle::new();
    oracle
        .account_value(deps, &env.contract.address)
        .map_err(Into::into)
}

pub fn query_base_asset(deps: Deps) -> ProxyResult<BaseAssetResponse> {
    let oracle = Oracle::new();
    let base_asset = oracle.base_asset(deps)?;
    Ok(BaseAssetResponse { base_asset })
}

pub fn query_holding_amount(
    deps: Deps,
    env: Env,
    identifier: AssetEntry,
) -> ProxyResult<HoldingAmountResponse> {
    let ans_host = ANS_HOST.load(deps.storage)?;
    let asset_info = identifier.resolve(&deps.querier, &ans_host)?;
    Ok(HoldingAmountResponse {
        amount: asset_info.query_balance(&deps.querier, env.contract.address)?,
    })
}

#[cfg(test)]
mod test {
    use abstract_os::objects::price_source::{PriceSource, UncheckedPriceSource};
    
    use abstract_sdk::feature_objects::AnsHost;
    use abstract_testing::{prelude::*, MockAnsHost};
    use cosmwasm_std::testing::mock_dependencies;
    use cosmwasm_std::testing::{
        mock_env, mock_info, MockApi, MockQuerier, MockStorage,
    };
    use cosmwasm_std::{Addr, Decimal, DepsMut, OwnedDeps};
    

    use abstract_os::proxy::{ExecuteMsg, InstantiateMsg};

    use crate::contract::{execute, instantiate, query};

    use super::*;

    const TEST_CREATOR: &str = "creator";

    type MockDeps = OwnedDeps<MockStorage, MockApi, MockQuerier>;

    pub fn get_ans() -> AnsHost {
        let addr = Addr::unchecked(TEST_ANS_HOST);
        let ans = AnsHost::new(addr);
        ans
    }

    pub fn base_asset() -> (AssetEntry, UncheckedPriceSource) {
        (AssetEntry::from(USD), UncheckedPriceSource::None)
    }

    pub fn asset_as_half() -> (AssetEntry, UncheckedPriceSource) {
        let asset = AssetEntry::from(EUR);
        let price_source = UncheckedPriceSource::ValueAs {
            asset: AssetEntry::new(USD),
            multiplier: Decimal::percent(50),
        };
        (asset, price_source)
    }

    fn mock_init(deps: DepsMut) {
        let info = mock_info(TEST_CREATOR, &[]);
        let msg = InstantiateMsg {
            os_id: 0,
            ans_host_address: TEST_ANS_HOST.to_string(),
        };
        let _res = instantiate(deps, mock_env(), info, msg).unwrap();
    }

    pub fn execute_as_admin(deps: &mut MockDeps, msg: ExecuteMsg) -> ProxyResult {
        let info = mock_info(TEST_CREATOR, &[]);
        execute(deps.as_mut(), mock_env(), info, msg)
    }

    #[test]
    fn query_base_asset() {
        let mut deps = mock_dependencies();
        deps.querier = MockAnsHost::new().with_defaults().to_querier();
        mock_init(deps.as_mut());
        execute_as_admin(
            &mut deps,
            ExecuteMsg::UpdateAssets {
                to_add: vec![base_asset()],
                to_remove: vec![],
            },
        )
        .unwrap();

        let base_asset: BaseAssetResponse = from_binary(
            &query(
                deps.as_ref(),
                mock_env(),
                abstract_os::proxy::QueryMsg::BaseAsset {},
            )
            .unwrap(),
        )
        .unwrap();
        assert_eq!(
            base_asset,
            BaseAssetResponse {
                base_asset: cw_asset::AssetInfoBase::Native(USD.to_string())
            }
        );
    }

    #[test]
    fn query_asset_configs() {
        let mut deps = mock_dependencies();
        deps.querier = MockAnsHost::new().with_defaults().to_querier();
        mock_init(deps.as_mut());
        execute_as_admin(
            &mut deps,
            ExecuteMsg::UpdateAssets {
                to_add: vec![base_asset(), asset_as_half()],
                to_remove: vec![],
            },
        )
        .unwrap();

        let assets: AssetsConfigResponse = from_binary(
            &query(
                deps.as_ref(),
                mock_env(),
                abstract_os::proxy::QueryMsg::AssetsConfig {
                    start_after: None,
                    limit: None,
                },
            )
            .unwrap(),
        )
        .unwrap();
        assert_eq!(
            assets,
            AssetsConfigResponse {
                assets: vec![asset_as_half(), base_asset()]
            }
        );
    }

    #[test]
    fn query_asset_infos() {
        let mut deps = mock_dependencies();
        deps.querier = MockAnsHost::new().with_defaults().to_querier();
        mock_init(deps.as_mut());
        execute_as_admin(
            &mut deps,
            ExecuteMsg::UpdateAssets {
                to_add: vec![base_asset(), asset_as_half()],
                to_remove: vec![],
            },
        )
        .unwrap();

        let assets: AssetsInfoResponse = from_binary(
            &query(
                deps.as_ref(),
                mock_env(),
                abstract_os::proxy::QueryMsg::AssetsInfo {
                    start_after: None,
                    limit: None,
                },
            )
            .unwrap(),
        )
        .unwrap();
        assert_eq!(
            assets,
            AssetsInfoResponse {
                assets: vec![
                    (
                        AssetInfo::native(EUR),
                        OracleAsset {
                            complexity: 1,
                            price_source: PriceSource::ValueAs {
                                asset: AssetInfo::native(USD),
                                multiplier: Decimal::percent(50),
                            },
                        }
                    ),
                    (
                        AssetInfo::native(USD),
                        OracleAsset {
                            complexity: 0,
                            price_source: PriceSource::None,
                        }
                    ),
                ]
            }
        );
    }
}
