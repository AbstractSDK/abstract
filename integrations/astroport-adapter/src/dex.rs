use abstract_dex_standard::Identify;

use crate::{ASTROPORT, AVAILABLE_CHAINS};
// Source https://github.com/astroport-fi/astroport-core
#[derive(Default)]
pub struct Astroport {}

impl Identify for Astroport {
    fn name(&self) -> &'static str {
        ASTROPORT
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
    abstract_sdk::std::objects::PoolAddress,
    astroport::pair::{PoolResponse, SimulationResponse},
    cosmwasm_std::{to_json_binary, wasm_execute, CosmosMsg, Decimal, Deps, Uint128},
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

#[cfg(feature = "full_integration")]
impl DexCommand for Astroport {
    fn swap(
        &self,
        _deps: Deps,
        pool_id: PoolAddress,
        offer_asset: Asset,
        _ask_asset: AssetInfo,
        belief_price: Option<Decimal>,
        max_spread: Option<Decimal>,
    ) -> Result<Vec<CosmosMsg>, DexError> {
        let pair_address = pool_id.expect_contract()?;

        let swap_msg: Vec<CosmosMsg> = match &offer_asset.info {
            AssetInfo::Native(_) => vec![wasm_execute(
                pair_address.to_string(),
                &astroport::pair::ExecuteMsg::Swap {
                    offer_asset: cw_asset_to_astroport(&offer_asset)?,
                    ask_asset_info: None,
                    belief_price,
                    max_spread,
                    to: None,
                },
                vec![offer_asset.clone().try_into()?],
            )?
            .into()],
            AssetInfo::Cw20(addr) => vec![wasm_execute(
                addr.to_string(),
                &Cw20ExecuteMsg::Send {
                    contract: pair_address.to_string(),
                    amount: offer_asset.amount,
                    msg: to_json_binary(&astroport::pair::Cw20HookMsg::Swap {
                        belief_price,
                        ask_asset_info: None,
                        max_spread,
                        to: None,
                    })?,
                },
                vec![],
            )?
            .into()],
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

        let mut astroport_assets = offer_assets
            .iter()
            .map(cw_asset_to_astroport)
            .collect::<Result<Vec<_>, _>>()?;

        // execute msg
        let msg = astroport::pair::ExecuteMsg::ProvideLiquidity {
            assets: vec![
                astroport_assets.swap_remove(0),
                astroport_assets.swap_remove(0),
            ],
            slippage_tolerance: max_spread,
            auto_stake: Some(false),
            receiver: None,
        };

        // approval msgs for cw20 tokens (if present)
        msgs.extend(cw_approve_msgs(&offer_assets, &pair_address)?);
        let coins = coins_in_assets(&offer_assets);

        // actual call to pair
        let liquidity_msg = wasm_execute(pair_address, &msg, coins)?.into();
        msgs.push(liquidity_msg);

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

        if paired_assets.len() > 1 {
            return Err(DexError::TooManyAssets(2));
        }
        // Get pair info
        let pair_config: PoolResponse = deps.querier.query_wasm_smart(
            pair_address.to_string(),
            &astroport::pair::QueryMsg::Pool {},
        )?;
        let astroport_offer_asset = cw_asset_to_astroport(&offer_asset)?;
        let other_asset = if pair_config.assets[0].info == astroport_offer_asset.info {
            let price =
                Decimal::from_ratio(pair_config.assets[1].amount, pair_config.assets[0].amount);
            let other_token_amount = price * offer_asset.amount;
            Asset {
                amount: other_token_amount,
                info: paired_assets[0].clone(),
            }
        } else if pair_config.assets[1].info == astroport_offer_asset.info {
            let price =
                Decimal::from_ratio(pair_config.assets[0].amount, pair_config.assets[1].amount);
            let other_token_amount = price * offer_asset.amount;
            Asset {
                amount: other_token_amount,
                info: paired_assets[0].clone(),
            }
        } else {
            return Err(DexError::ArgumentMismatch(
                offer_asset.to_string(),
                pair_config
                    .assets
                    .iter()
                    .map(|e| e.info.to_string())
                    .collect(),
            ));
        };

        let offer_assets = [offer_asset, other_asset];

        let coins = coins_in_assets(&offer_assets);

        // approval msgs for cw20 tokens (if present)
        let mut msgs = cw_approve_msgs(&offer_assets, &pair_address)?;

        // construct execute msg
        let astroport_assets = offer_assets
            .iter()
            .map(cw_asset_to_astroport)
            .collect::<Result<Vec<_>, _>>()?;

        let msg = astroport::pair::ExecuteMsg::ProvideLiquidity {
            assets: vec![astroport_assets[0].clone(), astroport_assets[1].clone()],
            slippage_tolerance: None,
            receiver: None,
            auto_stake: None,
        };

        // actual call to pair
        let liquidity_msg = wasm_execute(pair_address, &msg, coins)?.into();
        msgs.push(liquidity_msg);

        Ok(msgs)
    }

    fn withdraw_liquidity(
        &self,
        _deps: Deps,
        pool_id: PoolAddress,
        lp_token: Asset,
    ) -> Result<Vec<CosmosMsg>, DexError> {
        let pair_address = pool_id.expect_contract()?;

        let hook_msg = astroport::pair::Cw20HookMsg::WithdrawLiquidity { assets: vec![] };

        let withdraw_msg = lp_token.send_msg(pair_address, to_json_binary(&hook_msg)?)?;
        Ok(vec![withdraw_msg])
    }

    fn simulate_swap(
        &self,
        deps: Deps,
        pool_id: PoolAddress,
        offer_asset: Asset,
        _ask_asset: AssetInfo,
    ) -> Result<(Return, Spread, Fee, FeeOnInput), DexError> {
        let pair_address = pool_id.expect_contract()?;
        // Do simulation
        let SimulationResponse {
            return_amount,
            spread_amount,
            commission_amount,
        } = deps.querier.query_wasm_smart(
            pair_address.to_string(),
            &astroport::pair::QueryMsg::Simulation {
                offer_asset: cw_asset_to_astroport(&offer_asset)?,
                ask_asset_info: None,
            },
        )?;
        // commission paid in result asset
        Ok((return_amount, spread_amount, commission_amount, false))
    }
}

#[cfg(feature = "full_integration")]
fn cw_asset_to_astroport(asset: &Asset) -> Result<astroport::asset::Asset, DexError> {
    match &asset.info {
        AssetInfoBase::Native(denom) => Ok(astroport::asset::Asset {
            amount: asset.amount,
            info: astroport::asset::AssetInfo::NativeToken {
                denom: denom.clone(),
            },
        }),
        AssetInfoBase::Cw20(contract_addr) => Ok(astroport::asset::Asset {
            amount: asset.amount,
            info: astroport::asset::AssetInfo::Token {
                contract_addr: contract_addr.clone(),
            },
        }),
        _ => Err(DexError::UnsupportedAssetType(asset.info.to_string())),
    }
}

#[cfg(test)]
mod tests {
    use std::{assert_eq, str::FromStr};

    use abstract_dex_standard::tests::{expect_eq, DexCommandTester};
    use abstract_sdk::std::objects::PoolAddress;
    use cosmwasm_schema::serde::Deserialize;
    use cosmwasm_std::{
        coin, coins, from_json, to_json_binary, wasm_execute, Addr, Coin, CosmosMsg, Decimal,
        WasmMsg,
    };
    use cw20::Cw20ExecuteMsg;
    use cw_asset::{Asset, AssetInfo};
    use cw_orch::daemon::networks::PHOENIX_1;

    use super::Astroport;

    fn create_setup() -> DexCommandTester {
        DexCommandTester::new(PHOENIX_1.into(), Astroport {})
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
            CosmosMsg::Wasm(WasmMsg::Execute { msg, .. }) => from_json(msg).unwrap(),
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
                &astroport::pair::ExecuteMsg::Swap {
                    offer_asset: astroport::asset::Asset {
                        amount: amount.into(),
                        info: astroport::asset::AssetInfo::NativeToken {
                            denom: USDC.to_string(),
                        },
                    },
                    ask_asset_info: None,
                    belief_price: Some(Decimal::from_str("0.2").unwrap()),
                    max_spread: Some(max_spread()),
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
                &astroport::pair::ExecuteMsg::ProvideLiquidity {
                    assets: vec![
                        astroport::asset::Asset {
                            amount: amount_usdc.into(),
                            info: astroport::asset::AssetInfo::NativeToken {
                                denom: USDC.to_string(),
                            },
                        },
                        astroport::asset::Asset {
                            amount: amount_luna.into(),
                            info: astroport::asset::AssetInfo::NativeToken {
                                denom: LUNA.to_string(),
                            },
                        },
                    ],
                    slippage_tolerance: Some(max_spread()),
                    auto_stake: Some(false),
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
                &astroport::pair::ExecuteMsg::Swap {
                    offer_asset: astroport::asset::Asset {
                        amount: (amount_usdc / 2u128).into(),
                        info: astroport::asset::AssetInfo::NativeToken {
                            denom: USDC.to_string(),
                        },
                    },
                    ask_asset_info: None,
                    belief_price: None,
                    max_spread: Some(max_spread()),
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

        let unwrapped_msg: astroport::pair::ExecuteMsg = get_wasm_msg(msgs[0].clone());
        match unwrapped_msg {
            astroport::pair::ExecuteMsg::ProvideLiquidity {
                assets,
                slippage_tolerance,
                auto_stake,
                receiver,
            } => {
                assert_eq!(assets.len(), 2);
                assert_eq!(
                    assets[0],
                    astroport::asset::Asset {
                        amount: amount_usdc.into(),
                        info: astroport::asset::AssetInfo::NativeToken {
                            denom: USDC.to_string()
                        },
                    }
                );
                assert_eq!(slippage_tolerance, None);
                assert_eq!(auto_stake, None);
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
                    msg: to_json_binary(&astroport::pair::Cw20HookMsg::WithdrawLiquidity {
                        assets: vec![]
                    })
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
