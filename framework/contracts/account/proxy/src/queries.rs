use crate::contract::ProxyResult;

use abstract_core::objects::oracle::{AccountValue, Oracle};
use abstract_core::proxy::{
    AssetsConfigResponse, BaseAssetResponse, HoldingAmountResponse, OracleAsset, TokenValueResponse,
};
use abstract_sdk::core::objects::AssetEntry;
use abstract_sdk::core::proxy::state::{ANS_HOST, STATE};
use abstract_sdk::core::proxy::{AssetsInfoResponse, ConfigResponse};
use abstract_sdk::Resolve;
use cosmwasm_std::{Addr, Deps, Env, StdResult};
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
) -> ProxyResult<TokenValueResponse> {
    let oracle = Oracle::new();
    let ans_host = ANS_HOST.load(deps.storage)?;
    let asset_info = asset_entry.resolve(&deps.querier, &ans_host)?;
    let balance = asset_info.query_balance(&deps.querier, env.contract.address)?;
    let value = oracle.asset_value(deps, Asset::new(asset_info, balance))?;

    Ok(TokenValueResponse { value })
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
    use abstract_core::objects::price_source::{PriceSource, UncheckedPriceSource};

    use abstract_testing::{prelude::*, MockAnsHost};
    use cosmwasm_std::testing::{mock_dependencies, MOCK_CONTRACT_ADDR};
    use cosmwasm_std::testing::{mock_env, mock_info, MockApi, MockQuerier, MockStorage};
    use cosmwasm_std::{coin, Decimal, DepsMut, OwnedDeps};

    use abstract_core::proxy::{
        AssetConfigResponse, ExecuteMsg, InstantiateMsg, TokenValueResponse,
    };

    use crate::contract::{execute, instantiate, query};

    use super::*;

    const TEST_CREATOR: &str = "creator";

    type MockDeps = OwnedDeps<MockStorage, MockApi, MockQuerier>;

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
            account_id: 0,
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
                abstract_core::proxy::QueryMsg::BaseAsset {},
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
    fn query_config() {
        let mut deps = mock_dependencies();
        deps.querier = MockAnsHost::new().with_defaults().to_querier();
        mock_init(deps.as_mut());
        execute_as_admin(
            &mut deps,
            ExecuteMsg::AddModule {
                module: "test_module".to_string(),
            },
        )
        .unwrap();

        let config: ConfigResponse = from_binary(
            &query(
                deps.as_ref(),
                mock_env(),
                abstract_core::proxy::QueryMsg::Config {},
            )
            .unwrap(),
        )
        .unwrap();
        assert_eq!(
            config,
            ConfigResponse {
                modules: vec!["test_module".to_string()],
            }
        );
    }

    #[test]
    fn query_oracle() {
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

        // mint tokens to the contract
        deps.querier
            .update_balance(MOCK_CONTRACT_ADDR, vec![coin(1000, USD)]);

        // get the balance of the asset
        // returns HoldingAmountResponse
        let holding_amount: HoldingAmountResponse = from_binary(
            &query(
                deps.as_ref(),
                mock_env(),
                abstract_core::proxy::QueryMsg::HoldingAmount {
                    identifier: AssetEntry::from(USD),
                },
            )
            .unwrap(),
        )
        .unwrap();
        assert_eq!(holding_amount.amount.u128(), 1000);

        // get the value of the asset
        // returns AccountValue
        let account_value: AccountValue = from_binary(
            &query(
                deps.as_ref(),
                mock_env(),
                abstract_core::proxy::QueryMsg::TotalValue {},
            )
            .unwrap(),
        )
        .unwrap();
        // equal to balance as it's the base asset
        assert_eq!(
            account_value.total_value,
            Asset::new(AssetInfo::native(USD), 1000u128)
        );

        // get the token value
        // returns TokenValueResponse
        let token_value: TokenValueResponse = from_binary(
            &query(
                deps.as_ref(),
                mock_env(),
                abstract_core::proxy::QueryMsg::TokenValue {
                    identifier: AssetEntry::from(USD),
                },
            )
            .unwrap(),
        )
        .unwrap();
        assert_eq!(token_value.value.u128(), 1000u128);

        // query USD asset config
        let asset_config: AssetConfigResponse = from_binary(
            &query(
                deps.as_ref(),
                mock_env(),
                abstract_core::proxy::QueryMsg::AssetConfig {
                    identifier: AssetEntry::from(USD),
                },
            )
            .unwrap(),
        )
        .unwrap();
        assert_eq!(asset_config.price_source, UncheckedPriceSource::None);
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
                abstract_core::proxy::QueryMsg::AssetsConfig {
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
                abstract_core::proxy::QueryMsg::AssetsInfo {
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
