use crate::ASTROVAULT;
use crate::AVAILABLE_CHAINS;
use abstract_dex_standard::Identify;
use abstract_sdk::core::objects::PoolType;

use abstract_sdk::feature_objects::AnsHost;

#[derive(Default)]
pub struct Astrovault {
    pub pool_type: Option<PoolType>,
}

impl Identify for Astrovault {
    fn name(&self) -> &'static str {
        ASTROVAULT
    }
    fn is_available_on(&self, chain_name: &str) -> bool {
        AVAILABLE_CHAINS.contains(&chain_name)
    }
}

#[cfg(feature = "full_integration")]
use ::{
    abstract_dex_standard::{
        coins_in_assets, cw_approve_msgs, DexCommand, DexError, Fee, FeeOnInput, Return, Spread,
    },
    abstract_sdk::{
        core::objects::{PoolAddress, UniquePoolId},
        cw_helpers::wasm_smart_query,
        feature_objects::VersionControlContract,
        AbstractSdkResult,
    },
    cosmwasm_std::{to_binary, wasm_execute, CosmosMsg, Decimal, Deps, Uint128},
    cw20::Cw20ExecuteMsg,
    cw_asset::{Asset, AssetInfo, AssetInfoBase},
};

#[cfg(feature = "full_integration")]
/// This structure describes a CW20 hook message.
#[cosmwasm_schema::cw_serde]
pub enum StubCw20HookMsg {
    /// Withdraw liquidity from the pool
    WithdrawLiquidity {},
}

fn native_swap(
    deps: Deps,
    pool_type: PoolType,
    pair_address: cosmwasm_std::Addr,
    offer_asset: Asset,
    ask_asset: AssetInfo,
    belief_price: Option<Decimal>,
    max_spread: Option<Decimal>,
) -> Result<Vec<CosmosMsg>, DexError> {
    let msgs = match pool_type {
        PoolType::ConstantProduct => vec![wasm_execute(
            pair_address.to_string(),
            &astrovault::standard_pool::handle_msg::ExecuteMsg::Swap {
                offer_asset: cw_asset_to_astrovault(&offer_asset)?,
                belief_price,
                max_spread,
                expected_return: None,
                to: None,
            },
            vec![offer_asset.try_into()?],
        )?
        .into()],
        PoolType::Stable => {
            let pool_info: astrovault::assets::pools::PoolInfo = deps.querier.query_wasm_smart(
                pair_address.to_string(),
                &astrovault::stable_pool::query_msg::QueryMsg::PoolInfo {},
            )?;
            let ask_asset = cw_asset_info_to_astrovault(&ask_asset)?;
            let index = pool_info
                .asset_infos
                .iter()
                .position(|a| *a == ask_asset)
                .ok_or(DexError::ArgumentMismatch(
                    ask_asset.to_string(),
                    pool_info
                        .asset_infos
                        .into_iter()
                        .map(|a| a.to_string())
                        .collect(),
                ))?;
            vec![wasm_execute(
                pair_address.to_string(),
                &astrovault::stable_pool::handle_msg::ExecuteMsg::Swap {
                    swap_to_asset_index: index as u32,
                    expected_return: None,
                    to: None,
                },
                vec![offer_asset.try_into()?],
            )?
            .into()]
        }
        PoolType::Weighted => {
            vec![wasm_execute(
                pair_address.to_string(),
                &astrovault::ratio_pool::handle_msg::ExecuteMsg::Swap {
                    expected_return: None,
                    to: None,
                },
                vec![offer_asset.try_into()?],
            )?
            .into()]
        }
        _ => panic!("Unsupported pool type"),
    };
    Ok(msgs)
}

#[allow(clippy::too_many_arguments)]
fn cw20_swap(
    deps: Deps,
    cw20_addr: &cosmwasm_std::Addr,
    pool_type: PoolType,
    pair_address: cosmwasm_std::Addr,
    offer_asset: &Asset,
    ask_asset: AssetInfo,
    belief_price: Option<Decimal>,
    max_spread: Option<Decimal>,
) -> Result<Vec<CosmosMsg>, DexError> {
    let msgs = match pool_type {
        PoolType::ConstantProduct => vec![wasm_execute(
            cw20_addr.to_string(),
            &Cw20ExecuteMsg::Send {
                contract: pair_address.to_string(),
                amount: offer_asset.amount,
                msg: to_binary(&astrovault::standard_pool::handle_msg::Cw20HookMsg::Swap {
                    belief_price,
                    max_spread,
                    expected_return: None,
                    to: None,
                })?,
            },
            vec![],
        )?
        .into()],
        PoolType::Stable => {
            let pool_info: astrovault::assets::pools::PoolInfo = deps.querier.query_wasm_smart(
                pair_address.to_string(),
                &astrovault::stable_pool::query_msg::QueryMsg::PoolInfo {},
            )?;
            let ask_asset = cw_asset_info_to_astrovault(&ask_asset)?;
            let index = pool_info
                .asset_infos
                .iter()
                .position(|a| *a == ask_asset)
                .ok_or(DexError::ArgumentMismatch(
                    ask_asset.to_string(),
                    pool_info
                        .asset_infos
                        .into_iter()
                        .map(|a| a.to_string())
                        .collect(),
                ))?;
            vec![wasm_execute(
                cw20_addr.to_string(),
                &Cw20ExecuteMsg::Send {
                    contract: pair_address.to_string(),
                    amount: offer_asset.amount,
                    msg: to_binary(&astrovault::stable_pool::handle_msg::Cw20HookMsg::Swap {
                        swap_to_asset_index: index as u32,
                        expected_return: None,
                        to: None,
                    })?,
                },
                vec![],
            )?
            .into()]
        }
        PoolType::Weighted => {
            vec![wasm_execute(
                cw20_addr.to_string(),
                &Cw20ExecuteMsg::Send {
                    contract: pair_address.to_string(),
                    amount: offer_asset.amount,
                    msg: to_binary(&astrovault::ratio_pool::handle_msg::Cw20HookMsg::Swap {
                        expected_return: None,
                        to: None,
                    })?,
                },
                vec![],
            )?
            .into()]
        }
        _ => panic!("Unsupported pool type"),
    };
    Ok(msgs)
}

#[cfg(feature = "full_integration")]
impl DexCommand for Astrovault {
    fn fetch_data(
        &mut self,
        deps: Deps,
        _sender: cosmwasm_std::Addr,
        _version_control_contract: VersionControlContract,
        ans_host: AnsHost,
        pool_id: UniquePoolId,
    ) -> AbstractSdkResult<()> {
        let pool_metadata = ans_host.query_pool_metadata(&deps.querier, &pool_id)?;
        self.pool_type = Some(pool_metadata.pool_type);
        Ok(())
    }

    fn swap(
        &self,
        deps: Deps,
        pool_id: PoolAddress,
        offer_asset: Asset,
        ask_asset: AssetInfo,
        belief_price: Option<Decimal>,
        max_spread: Option<Decimal>,
    ) -> Result<Vec<CosmosMsg>, DexError> {
        let pair_address = pool_id.expect_contract()?;

        let pool_type = self.pool_type.unwrap();
        let swap_msg: Vec<CosmosMsg> = match &offer_asset.info {
            AssetInfo::Native(_) => native_swap(
                deps,
                pool_type,
                pair_address,
                offer_asset,
                ask_asset,
                belief_price,
                max_spread,
            )?,
            AssetInfo::Cw20(addr) => cw20_swap(
                deps,
                addr,
                pool_type,
                pair_address,
                &offer_asset,
                ask_asset,
                belief_price,
                max_spread,
            )?,
            _ => panic!("unsupported asset"),
        };
        Ok(swap_msg)
    }

    fn provide_liquidity(
        &self,
        deps: Deps,
        pool_id: PoolAddress,
        mut offer_assets: Vec<Asset>,
        max_spread: Option<Decimal>,
    ) -> Result<Vec<CosmosMsg>, DexError> {
        let pair_address = pool_id.expect_contract()?;
        let mut msgs = vec![];

        // We know that (+)two assets were provided because it's a requirement to resolve the pool
        // We don't know if one of the asset amounts is 0, which would require a simulation and swap before providing liquidity
        if offer_assets.len() > 2 {
            return Err(DexError::TooManyAssets(2));
        } else if offer_assets.iter().any(|a| a.amount.is_zero()) {
            // find 0 asset
            let (index, non_zero_offer_asset) = offer_assets
                .iter()
                .enumerate()
                .find(|(_, a)| !a.amount.is_zero())
                .ok_or(DexError::TooFewAssets {})?;

            // the other asset in offer_assets is the one with amount zero
            let ask_asset = offer_assets.get((index + 1) % 2).unwrap().info.clone();

            // we want to offer half of the non-zero asset to swap into the ask asset
            let offer_asset = Asset::new(
                non_zero_offer_asset.info.clone(),
                non_zero_offer_asset
                    .amount
                    .checked_div(Uint128::from(2u128))
                    .unwrap(),
            );

            // simulate swap to get the amount of ask asset we can provide after swapping
            let simulated_received = self
                .simulate_swap(
                    deps,
                    pool_id.clone(),
                    offer_asset.clone(),
                    ask_asset.clone(),
                )?
                .0;
            let swap_msg = self.swap(
                deps,
                pool_id,
                offer_asset.clone(),
                ask_asset.clone(),
                None,
                max_spread,
            )?;
            // add swap msg
            msgs.extend(swap_msg);
            // update the offer assets for providing liquidity
            offer_assets = vec![offer_asset, Asset::new(ask_asset, simulated_received)];
        }

        let mut astrovault_assets = offer_assets
            .iter()
            .map(cw_asset_to_astrovault)
            .collect::<Result<Vec<_>, _>>()?;

        // approval msgs for cw20 tokens (if present)
        msgs.extend(cw_approve_msgs(&offer_assets, &pair_address)?);
        let coins = coins_in_assets(&offer_assets);
        // execute msg
        let liquidity_msg = match self.pool_type.unwrap() {
            PoolType::ConstantProduct => wasm_execute(
                pair_address,
                &astrovault::standard_pool::handle_msg::ExecuteMsg::ProvideLiquidity {
                    assets: [
                        astrovault_assets.swap_remove(0),
                        astrovault_assets.swap_remove(0),
                    ],
                    slippage_tolerance: max_spread,
                    direct_staking: None,
                    receiver: None,
                },
                coins,
            )?,
            PoolType::Stable => wasm_execute(
                pair_address,
                &astrovault::stable_pool::handle_msg::ExecuteMsg::Deposit {
                    // TODO: it can be >2
                    assets_amount: vec![
                        astrovault_assets.swap_remove(0).amount,
                        astrovault_assets.swap_remove(0).amount,
                    ],
                    direct_staking: None,
                    receiver: None,
                },
                coins,
            )?,
            PoolType::Weighted => wasm_execute(
                pair_address,
                &astrovault::ratio_pool::handle_msg::ExecuteMsg::Deposit {
                    assets_amount: [
                        astrovault_assets.swap_remove(0).amount,
                        astrovault_assets.swap_remove(0).amount,
                    ],
                    direct_staking: None,
                    receiver: None,
                    expected_return: None,
                },
                coins,
            )?,
            _ => panic!("Unsupported pool type"),
        };

        // actual call to pair
        msgs.push(liquidity_msg.into());

        Ok(msgs)
    }

    fn provide_liquidity_symmetric(
        &self,
        deps: Deps,
        pool_id: PoolAddress,
        offer_asset: Asset,
        paired_assets: Vec<AssetInfo>,
    ) -> Result<Vec<CosmosMsg>, DexError> {
        let pair_address = pool_id.expect_contract()?;

        // TODO: cover three-way stable
        if paired_assets.len() > 1 {
            return Err(DexError::TooManyAssets(2));
        }
        // Get pair info
        let pair_assets = match self.pool_type.unwrap() {
            PoolType::ConstantProduct => {
                let pool_response: astrovault::standard_pool::query_msg::PoolResponse =
                    deps.querier.query(&wasm_smart_query(
                        pair_address.to_string(),
                        &astrovault::standard_pool::query_msg::QueryMsg::Pool {},
                    )?)?;
                pool_response.assets.to_vec()
            }
            PoolType::Stable => {
                let pool_response: astrovault::stable_pool::query_msg::PoolResponse =
                    deps.querier.query(&wasm_smart_query(
                        pair_address.to_string(),
                        &astrovault::stable_pool::query_msg::QueryMsg::Pool {},
                    )?)?;
                pool_response.assets
            }
            PoolType::Weighted => {
                let pool_response: astrovault::ratio_pool::query_msg::PoolResponse =
                    deps.querier.query(&wasm_smart_query(
                        pair_address.to_string(),
                        &astrovault::ratio_pool::query_msg::QueryMsg::Pool {},
                    )?)?;
                pool_response.assets.to_vec()
            }
            _ => panic!("Unsupported pool type"),
        };
        let astrovault_offer_asset = cw_asset_to_astrovault(&offer_asset)?;
        let other_asset = if pair_assets[0].info == astrovault_offer_asset.info {
            let price = Decimal::from_ratio(pair_assets[1].amount, pair_assets[0].amount);
            let other_token_amount = price * offer_asset.amount;
            Asset {
                amount: other_token_amount,
                info: paired_assets[0].clone(),
            }
        } else if pair_assets[1].info == astrovault_offer_asset.info {
            let price = Decimal::from_ratio(pair_assets[0].amount, pair_assets[1].amount);
            let other_token_amount = price * offer_asset.amount;
            Asset {
                amount: other_token_amount,
                info: paired_assets[0].clone(),
            }
        } else {
            return Err(DexError::ArgumentMismatch(
                offer_asset.to_string(),
                pair_assets.iter().map(|e| e.info.to_string()).collect(),
            ));
        };

        let offer_assets = [offer_asset, other_asset];

        let coins = coins_in_assets(&offer_assets);

        // approval msgs for cw20 tokens (if present)
        let mut msgs = cw_approve_msgs(&offer_assets, &pair_address)?;

        // construct execute msg
        let astrovault_assets = offer_assets
            .iter()
            .map(cw_asset_to_astrovault)
            .collect::<Result<Vec<_>, _>>()?;

        let liquidity_msg = match self.pool_type.unwrap() {
            PoolType::ConstantProduct => wasm_execute(
                pair_address,
                &astrovault::standard_pool::handle_msg::ExecuteMsg::ProvideLiquidity {
                    assets: [astrovault_assets[0].clone(), astrovault_assets[1].clone()],
                    slippage_tolerance: None,
                    direct_staking: None,
                    receiver: None,
                },
                coins,
            )?,
            PoolType::Stable => wasm_execute(
                pair_address,
                &astrovault::stable_pool::handle_msg::ExecuteMsg::Deposit {
                    assets_amount: vec![astrovault_assets[0].amount, astrovault_assets[1].amount],
                    direct_staking: None,
                    receiver: None,
                },
                coins,
            )?,
            PoolType::Weighted => wasm_execute(
                pair_address,
                &astrovault::ratio_pool::handle_msg::ExecuteMsg::Deposit {
                    assets_amount: [astrovault_assets[0].amount, astrovault_assets[1].amount],
                    direct_staking: None,
                    receiver: None,
                    expected_return: None,
                },
                coins,
            )?,
            _ => panic!("Unsupported pool type"),
        };

        // actual call to pair
        msgs.push(liquidity_msg.into());

        Ok(msgs)
    }

    fn withdraw_liquidity(
        &self,
        _deps: Deps,
        pool_id: PoolAddress,
        lp_token: Asset,
    ) -> Result<Vec<CosmosMsg>, DexError> {
        let pair_address = pool_id.expect_contract()?;

        let hook_msg = match self.pool_type.unwrap() {
            PoolType::ConstantProduct => to_binary(
                &astrovault::standard_pool::handle_msg::Cw20HookMsg::WithdrawLiquidity(
                    astrovault::standard_pool::handle_msg::WithdrawLiquidityInputs { to: None },
                ),
            )?,
            PoolType::Stable => to_binary(
                &astrovault::stable_pool::handle_msg::Cw20HookMsg::WithdrawalToLockup(
                    astrovault::stable_pool::handle_msg::WithdrawalToLockupInputs {
                        // TODO:
                        withdrawal_lockup_assets_amount: vec![],
                        to: None,
                        is_instant_withdrawal: Some(true),
                        expected_return: None,
                    },
                ),
            )?,
            PoolType::Weighted => to_binary(
                &astrovault::ratio_pool::handle_msg::Cw20HookMsg::WithdrawalToLockup(
                    astrovault::ratio_pool::handle_msg::RatioWithdrawalToLockupInputs {
                        to: None,
                        is_instant_withdrawal: Some(true),
                        expected_return: None,
                    },
                ),
            )?,
            _ => panic!("Unsupported pool type"),
        };

        let withdraw_msg = lp_token.send_msg(pair_address, hook_msg)?;
        Ok(vec![withdraw_msg])
    }

    fn simulate_swap(
        &self,
        deps: Deps,
        pool_id: PoolAddress,
        offer_asset: Asset,
        ask_asset: AssetInfo,
    ) -> Result<(Return, Spread, Fee, FeeOnInput), DexError> {
        let pair_address = pool_id.expect_contract()?;
        // Do simulation
        match self.pool_type.unwrap() {
            PoolType::ConstantProduct => {
                let astrovault::standard_pool::query_msg::SimulationResponse {
                    return_amount,
                    spread_amount,
                    commission_amount,
                    buybackburn_amount: _,
                } = deps.querier.query(&wasm_smart_query(
                    pair_address.to_string(),
                    &astrovault::standard_pool::query_msg::QueryMsg::Simulation {
                        offer_asset: cw_asset_to_astrovault(&offer_asset)?,
                    },
                )?)?;
                // commission paid in result asset
                Ok((return_amount, spread_amount, commission_amount, false))
            }
            PoolType::Stable => {
                let pool_info: astrovault::assets::pools::PoolInfo =
                    deps.querier.query_wasm_smart(
                        pair_address.to_string(),
                        &astrovault::stable_pool::query_msg::QueryMsg::PoolInfo {},
                    )?;
                let ask_astrovault_asset = cw_asset_info_to_astrovault(&ask_asset)?;
                let ask_index = pool_info
                    .asset_infos
                    .iter()
                    .position(|a| *a == ask_astrovault_asset)
                    .ok_or(DexError::ArgumentMismatch(
                        ask_astrovault_asset.to_string(),
                        pool_info
                            .asset_infos
                            .iter()
                            .map(ToString::to_string)
                            .collect(),
                    ))?;
                let offer_astrovault_asset = cw_asset_info_to_astrovault(&offer_asset.info)?;
                let offer_index = pool_info
                    .asset_infos
                    .iter()
                    .position(|a| *a == offer_astrovault_asset)
                    .ok_or(DexError::ArgumentMismatch(
                        ask_astrovault_asset.to_string(),
                        pool_info
                            .asset_infos
                            .iter()
                            .map(ToString::to_string)
                            .collect(),
                    ))?;
                let astrovault::standard_pool::query_msg::SimulationResponse {
                    return_amount,
                    spread_amount,
                    commission_amount,
                    buybackburn_amount: _,
                } = deps.querier.query(&wasm_smart_query(
                    pair_address.to_string(),
                    &astrovault::stable_pool::query_msg::QueryMsg::SwapSimulation {
                        amount: offer_asset.amount,
                        swap_from_asset_index: offer_index as u32,
                        swap_to_asset_index: ask_index as u32,
                    },
                )?)?;
                // commission paid in result asset
                Ok((return_amount, spread_amount, commission_amount, false))
            }
            PoolType::Weighted => todo!(),
            _ => panic!("Unsupported pool type"),
        }
    }
}

#[cfg(feature = "full_integration")]
fn cw_asset_to_astrovault(asset: &Asset) -> Result<astrovault::assets::asset::Asset, DexError> {
    match &asset.info {
        AssetInfoBase::Native(denom) => Ok(astrovault::assets::asset::Asset {
            amount: asset.amount,
            info: astrovault::assets::asset::AssetInfo::NativeToken {
                denom: denom.clone(),
            },
        }),
        AssetInfoBase::Cw20(contract_addr) => Ok(astrovault::assets::asset::Asset {
            amount: asset.amount,
            info: astrovault::assets::asset::AssetInfo::Token {
                contract_addr: contract_addr.to_string(),
            },
        }),
        _ => Err(DexError::UnsupportedAssetType(asset.info.to_string())),
    }
}

#[cfg(feature = "full_integration")]
fn cw_asset_info_to_astrovault(
    info: &AssetInfo,
) -> Result<astrovault::assets::asset::AssetInfo, DexError> {
    match &info {
        AssetInfoBase::Native(denom) => Ok(astrovault::assets::asset::AssetInfo::NativeToken {
            denom: denom.clone(),
        }),
        AssetInfoBase::Cw20(contract_addr) => Ok(astrovault::assets::asset::AssetInfo::Token {
            contract_addr: contract_addr.to_string(),
        }),
        _ => Err(DexError::UnsupportedAssetType(info.to_string())),
    }
}

#[cfg(test)]
mod tests {
    use abstract_dex_standard::tests::expect_eq;
    use cosmwasm_schema::serde::Deserialize;
    use cosmwasm_std::to_binary;
    use cosmwasm_std::Coin;

    use cosmwasm_std::coin;
    use cosmwasm_std::from_binary;
    use cosmwasm_std::CosmosMsg;
    use cosmwasm_std::WasmMsg;
    use cw20::Cw20ExecuteMsg;

    use super::Astrovault;
    use abstract_dex_standard::tests::DexCommandTester;
    use abstract_sdk::core::objects::PoolAddress;
    use cosmwasm_std::coins;
    use cosmwasm_std::Decimal;
    use cosmwasm_std::{wasm_execute, Addr};
    use cw_asset::{Asset, AssetInfo};
    use cw_orch::daemon::networks::PHOENIX_1;
    use std::assert_eq;
    use std::str::FromStr;

    fn create_setup() -> DexCommandTester {
        DexCommandTester::new(PHOENIX_1.into(), Astrovault { pool_type: None })
    }

    const POOL_CONTRACT: &str = "terra1fd68ah02gr2y8ze7tm9te7m70zlmc7vjyyhs6xlhsdmqqcjud4dql4wpxr";
    const LP_TOKEN: &str = "terra1ckmsqdhlky9jxcmtyj64crgzjxad9pvsd58k8zsxsnv4vzvwdt7qke04hl";
    const USDC: &str = "ibc/B3504E092456BA618CC28AC671A71FB08C6CA0FD0BE7C8A5B5A3E2DD933CC9E4";
    const LUNA: &str = "uluna";

    fn max_spread() -> Decimal {
        Decimal::from_str("0.1").unwrap()
    }

    fn get_wasm_msg<T: for<'de> Deserialize<'de>>(msg: CosmosMsg) -> T {
        match msg {
            CosmosMsg::Wasm(WasmMsg::Execute { msg, .. }) => from_binary(&msg).unwrap(),
            _ => panic!("Expected execute wasm msg, got a different enum"),
        }
    }

    fn get_wasm_addr(msg: CosmosMsg) -> String {
        match msg {
            CosmosMsg::Wasm(WasmMsg::Execute { contract_addr, .. }) => contract_addr,
            _ => panic!("Expected execute wasm msg, got a different enum"),
        }
    }

    fn get_wasm_funds(msg: CosmosMsg) -> Vec<Coin> {
        match msg {
            CosmosMsg::Wasm(WasmMsg::Execute { funds, .. }) => funds,
            _ => panic!("Expected execute wasm msg, got a different enum"),
        }
    }

    #[test]
    fn swap() {
        let amount = 100_000u128;
        let msgs = create_setup()
            .test_swap(
                PoolAddress::contract(Addr::unchecked(POOL_CONTRACT)),
                Asset::new(AssetInfo::native(USDC), amount),
                AssetInfo::native(LUNA),
                Some(Decimal::from_str("0.2").unwrap()),
                Some(max_spread()),
            )
            .unwrap();

        expect_eq(
            vec![wasm_execute(
                POOL_CONTRACT,
                &astrovault::standard_pool::handle_msg::ExecuteMsg::Swap {
                    offer_asset: astrovault::assets::asset::Asset {
                        amount: amount.into(),
                        info: astrovault::assets::asset::AssetInfo::NativeToken {
                            denom: USDC.to_string(),
                        },
                    },
                    belief_price: Some(Decimal::from_str("0.2").unwrap()),
                    max_spread: Some(max_spread()),
                    expected_return: None,
                    to: None,
                },
                coins(amount, USDC),
            )
            .unwrap()
            .into()],
            msgs,
        )
        .unwrap();
    }

    #[test]
    fn provide_liquidity() {
        let amount_usdc = 100_000u128;
        let amount_luna = 50_000u128;
        let msgs = create_setup()
            .test_provide_liquidity(
                PoolAddress::contract(Addr::unchecked(POOL_CONTRACT)),
                vec![
                    Asset::new(AssetInfo::native(USDC), amount_usdc),
                    Asset::new(AssetInfo::native(LUNA), amount_luna),
                ],
                Some(max_spread()),
            )
            .unwrap();

        expect_eq(
            vec![wasm_execute(
                POOL_CONTRACT,
                &astrovault::standard_pool::handle_msg::ExecuteMsg::ProvideLiquidity {
                    assets: [
                        astrovault::assets::asset::Asset {
                            amount: amount_usdc.into(),
                            info: astrovault::assets::asset::AssetInfo::NativeToken {
                                denom: USDC.to_string(),
                            },
                        },
                        astrovault::assets::asset::Asset {
                            amount: amount_luna.into(),
                            info: astrovault::assets::asset::AssetInfo::NativeToken {
                                denom: LUNA.to_string(),
                            },
                        },
                    ],
                    slippage_tolerance: Some(max_spread()),
                    direct_staking: None,
                    receiver: None,
                },
                vec![coin(amount_usdc, USDC), coin(amount_luna, LUNA)],
            )
            .unwrap()
            .into()],
            msgs,
        )
        .unwrap();
    }

    #[test]
    fn provide_liquidity_one_side() {
        let amount_usdc = 100_000u128;
        let amount_luna = 0u128;
        let msgs = create_setup()
            .test_provide_liquidity(
                PoolAddress::contract(Addr::unchecked(POOL_CONTRACT)),
                vec![
                    Asset::new(AssetInfo::native(USDC), amount_usdc),
                    Asset::new(AssetInfo::native(LUNA), amount_luna),
                ],
                Some(max_spread()),
            )
            .unwrap();

        // There should be a swap before providing liquidity
        // We can't really test much further, because this unit test is querying mainnet liquidity pools
        expect_eq(
            wasm_execute(
                POOL_CONTRACT,
                &astrovault::standard_pool::handle_msg::ExecuteMsg::Swap {
                    offer_asset: astrovault::assets::asset::Asset {
                        amount: (amount_usdc / 2u128).into(),
                        info: astrovault::assets::asset::AssetInfo::NativeToken {
                            denom: USDC.to_string(),
                        },
                    },
                    belief_price: None,
                    max_spread: Some(max_spread()),
                    expected_return: None,
                    to: None,
                },
                coins(amount_usdc / 2u128, USDC),
            )
            .unwrap()
            .into(),
            msgs[0].clone(),
        )
        .unwrap();
    }

    #[test]
    fn provide_liquidity_symmetric() {
        let amount_usdc = 100_000u128;
        let msgs = create_setup()
            .test_provide_liquidity_symmetric(
                PoolAddress::contract(Addr::unchecked(POOL_CONTRACT)),
                Asset::new(AssetInfo::native(USDC), amount_usdc),
                vec![AssetInfo::native(LUNA)],
            )
            .unwrap();

        assert_eq!(msgs.len(), 1);
        assert_eq!(get_wasm_addr(msgs[0].clone()), POOL_CONTRACT);

        let unwrapped_msg: astrovault::standard_pool::handle_msg::ExecuteMsg =
            get_wasm_msg(msgs[0].clone());
        match unwrapped_msg {
            astrovault::standard_pool::handle_msg::ExecuteMsg::ProvideLiquidity {
                assets,
                slippage_tolerance,
                receiver,
                direct_staking,
            } => {
                assert_eq!(assets.len(), 2);
                assert_eq!(
                    assets[0],
                    astrovault::assets::asset::Asset {
                        amount: amount_usdc.into(),
                        info: astrovault::assets::asset::AssetInfo::NativeToken {
                            denom: USDC.to_string()
                        },
                    }
                );
                assert_eq!(slippage_tolerance, None);
                assert_eq!(direct_staking, None);
                assert_eq!(receiver, None)
            }
            _ => panic!("Expected a provide liquidity variant"),
        }

        let funds = get_wasm_funds(msgs[0].clone());
        assert_eq!(funds.len(), 2);
        assert_eq!(funds[0], coin(amount_usdc, USDC),);
    }

    #[test]
    fn withdraw_liquidity() {
        let amount_lp = 100_000u128;
        let msgs = create_setup()
            .test_withdraw_liquidity(
                PoolAddress::contract(Addr::unchecked(POOL_CONTRACT)),
                Asset::new(AssetInfo::cw20(Addr::unchecked(LP_TOKEN)), amount_lp),
            )
            .unwrap();

        assert_eq!(
            msgs,
            vec![wasm_execute(
                LP_TOKEN,
                &Cw20ExecuteMsg::Send {
                    contract: POOL_CONTRACT.to_string(),
                    amount: amount_lp.into(),
                    msg: to_binary(
                        &astrovault::standard_pool::handle_msg::Cw20HookMsg::WithdrawLiquidity(
                            astrovault::standard_pool::handle_msg::WithdrawLiquidityInputs {
                                to: None
                            }
                        )
                    )
                    .unwrap()
                },
                vec![]
            )
            .unwrap()
            .into()]
        );
    }

    #[test]
    fn simulate_swap() {
        let amount = 100_000u128;
        // We siply verify it's executed, no check on what is returned
        create_setup()
            .test_simulate_swap(
                PoolAddress::contract(Addr::unchecked(POOL_CONTRACT)),
                Asset::new(AssetInfo::native(USDC), amount),
                AssetInfo::native(LUNA),
            )
            .unwrap();
    }
}
