use abstract_dex_standard::Identify;

use crate::AVAILABLE_CHAINS;

pub const FIN: &str = "fin";

// Source https://docs.rs/kujira/0.8.2/kujira/
#[derive(Default)]
pub struct Fin {}

impl Identify for Fin {
    fn name(&self) -> &'static str {
        FIN
    }
    fn is_available_on(&self, chain_name: &str) -> bool {
        AVAILABLE_CHAINS.contains(&chain_name)
    }
}

#[cfg(feature = "full_integration")]
use ::{
    abstract_dex_standard::{
        coins_in_assets, DexCommand, DexError, Fee, FeeOnInput, Return, Spread,
    },
    abstract_sdk::std::objects::PoolAddress,
    cosmwasm_std::{
        wasm_execute, Addr, Coin, CosmosMsg, Decimal, Decimal256, Deps, StdError, StdResult,
        Uint128,
    },
    cw_asset::{Asset, AssetInfo, AssetInfoBase},
    kujira::{
        bow::{
            self,
            market_maker::{ConfigResponse, PoolResponse},
        },
        fin,
    },
};

#[cfg(feature = "full_integration")]
impl DexCommand for Fin {
    fn swap(
        &self,
        _deps: Deps,
        pool_id: PoolAddress,
        offer_asset: Asset,
        _ask_asset: AssetInfo,
        belief_price: Option<Decimal>,
        max_spread: Option<Decimal>,
    ) -> Result<Vec<CosmosMsg>, DexError> {
        let fin_pair_address: Addr = match pool_id {
            PoolAddress::SeparateAddresses { swap, liquidity: _ } => swap,
            PoolAddress::Contract(swap) => swap,
            _ => panic!("invalid address"),
        };

        let swap_msg: Vec<CosmosMsg> = match &offer_asset.info {
            AssetInfo::Native(_) => vec![wasm_execute(
                fin_pair_address.to_string(),
                &fin::ExecuteMsg::Swap {
                    offer_asset: Some(Coin::try_from(&offer_asset)?),
                    belief_price: if let Some(belief_price) = belief_price {
                        Some(decimal2decimal256(belief_price)?)
                    } else {
                        None
                    },
                    max_spread: if let Some(max_spread) = max_spread {
                        Some(decimal2decimal256(max_spread)?)
                    } else {
                        None
                    },
                    to: None,
                    callback: None,
                },
                vec![offer_asset.clone().try_into()?],
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
        let bow_pair_address = match &pool_id {
            PoolAddress::SeparateAddresses { swap: _, liquidity } => liquidity,
            PoolAddress::Contract(liquidity) => liquidity,
            _ => panic!("invalid address"),
        };
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
                pool_id.clone(),
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

        // execute msg
        let msg = bow::market_maker::ExecuteMsg::Deposit {
            max_slippage: max_spread,
            callback: None,
        };

        let coins = coins_in_assets(&offer_assets);

        // actual call to pair
        let liquidity_msg = wasm_execute(bow_pair_address, &msg, coins)?.into();
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
        // unimplemented!();
        let bow_pair_address: Addr = match pool_id {
            PoolAddress::SeparateAddresses { swap: _, liquidity } => liquidity,
            PoolAddress::Contract(liquidity) => liquidity,
            _ => panic!("invalid address"),
        };
        let mut msgs = vec![];

        if paired_assets.len() > 1 {
            return Err(DexError::TooManyAssets(2));
        }

        // Pair config
        let pair_config: ConfigResponse = deps.querier.query_wasm_smart(
            bow_pair_address.to_string(),
            &bow::market_maker::QueryMsg::Config {},
        )?;

        // Get pair info
        let pair_info: PoolResponse = deps.querier.query_wasm_smart(
            bow_pair_address.to_string(),
            &bow::market_maker::QueryMsg::Pool {},
        )?;

        let pair_assets: Vec<kujira::Asset> = vec![
            kujira::Asset {
                amount: pair_info.balances[0],
                info: kujira::AssetInfo::NativeToken {
                    denom: pair_config.denoms[0].clone(),
                },
            },
            kujira::Asset {
                amount: pair_info.balances[1],
                info: kujira::AssetInfo::NativeToken {
                    denom: pair_config.denoms[1].clone(),
                },
            },
        ];
        let kujira_offer_asset = cw_asset_to_kujira(&offer_asset)?;
        let other_asset = if pair_assets[0].info == kujira_offer_asset.info {
            let price = Decimal::from_ratio(pair_assets[1].amount, pair_assets[0].amount);
            let other_token_amount = price * offer_asset.amount;
            Asset {
                amount: other_token_amount,
                info: paired_assets[0].clone(),
            }
        } else if pair_assets[1].info == kujira_offer_asset.info {
            let price = Decimal::from_ratio(pair_assets[0].amount, pair_assets[1].amount);
            let other_token_amount = price * offer_asset.amount;
            Asset {
                amount: other_token_amount,
                info: paired_assets[0].clone(),
            }
        } else {
            return Err(DexError::ArgumentMismatch(
                offer_asset.to_string(),
                pair_config.denoms.iter().map(|e| e.to_string()).collect(),
            ));
        };

        let offer_assets = [offer_asset, other_asset];

        let coins = coins_in_assets(&offer_assets);

        let msg = bow::market_maker::ExecuteMsg::Deposit {
            max_slippage: None,
            callback: None,
        };

        // actual call to pair
        let liquidity_msg = wasm_execute(bow_pair_address, &msg, coins)?.into();
        msgs.push(liquidity_msg);

        Ok(msgs)
    }

    fn withdraw_liquidity(
        &self,
        _deps: Deps,
        pool_id: PoolAddress,
        lp_token: Asset,
    ) -> Result<Vec<CosmosMsg>, DexError> {
        let bow_pair_address: Addr = match pool_id {
            PoolAddress::SeparateAddresses { swap: _, liquidity } => liquidity,
            PoolAddress::Contract(liquidity) => liquidity,
            _ => panic!("invalid address"),
        };

        // execute msg
        let msg = bow::market_maker::ExecuteMsg::Withdraw { callback: None };
        let funds = vec![Coin::try_from(lp_token)?];
        let withdraw_msg = wasm_execute(bow_pair_address, &msg, funds)?.into();
        Ok(vec![withdraw_msg])
    }

    fn simulate_swap(
        &self,
        deps: Deps,
        pool_id: PoolAddress,
        offer_asset: Asset,
        _ask_asset: AssetInfo,
    ) -> Result<(Return, Spread, Fee, FeeOnInput), DexError> {
        let fin_pair_address: Addr = match pool_id {
            PoolAddress::SeparateAddresses { swap, liquidity: _ } => swap,
            PoolAddress::Contract(swap) => swap,
            _ => panic!("invalid address"),
        };
        // Do simulation
        let fin::SimulationResponse {
            return_amount,
            spread_amount,
            commission_amount,
        } = deps.querier.query_wasm_smart(
            fin_pair_address.to_string(),
            &fin::QueryMsg::Simulation {
                offer_asset: cw_asset_to_kujira(&offer_asset)?,
            },
        )?;
        // commission paid in result asset
        Ok((
            Uint128::try_from(return_amount).unwrap(),
            Uint128::try_from(spread_amount).unwrap(),
            Uint128::try_from(commission_amount).unwrap(),
            false,
        ))
    }
}

#[cfg(feature = "full_integration")]
fn cw_asset_to_kujira(asset: &Asset) -> Result<kujira::Asset, DexError> {
    match &asset.info {
        AssetInfoBase::Native(denom) => Ok(kujira::Asset {
            amount: asset.amount,
            info: kujira::AssetInfo::NativeToken {
                denom: denom.into(),
            },
        }),
        _ => Err(DexError::UnsupportedAssetType(asset.info.to_string())),
    }
}

#[cfg(feature = "full_integration")]
/// Converts [`Decimal`] to [`Decimal256`].
pub fn decimal2decimal256(dec_value: Decimal) -> StdResult<Decimal256> {
    Decimal256::from_atomics(dec_value.atomics(), dec_value.decimal_places()).map_err(|_| {
        StdError::generic_err(format!(
            "Failed to convert Decimal {} to Decimal256",
            dec_value
        ))
    })
}

#[cfg(test)]
mod tests {
    use std::{assert_eq, str::FromStr};

    use abstract_dex_standard::tests::{expect_eq, DexCommandTester};
    use abstract_sdk::std::objects::PoolAddress;
    use cosmwasm_schema::serde::Deserialize;
    use cosmwasm_std::{
        coin, coins, from_json, wasm_execute, Addr, Coin, CosmosMsg, Decimal, Decimal256, WasmMsg,
    };
    use cw_asset::{Asset, AssetInfo};
    use cw_orch::daemon::networks::HARPOON_4;
    use kujira::{bow, fin};

    use super::{decimal2decimal256, Fin};

    fn create_setup() -> DexCommandTester {
        DexCommandTester::new(HARPOON_4.into(), Fin {})
    }

    const POOL_CONTRACT: &str = "kujira19kxd9sqk09zlzqfykk7tzyf70hl009hkekufq8q0ud90ejtqvvxs8xg5cq";
    const SWAP_CONTRACT: &str = "kujira1suhgf5svhu4usrurvxzlgn54ksxmn8gljarjtxqnapv8kjnp4nrsqq4jjh";
    const LP_TOKEN: &str =
        "factory/kujira19kxd9sqk09zlzqfykk7tzyf70hl009hkekufq8q0ud90ejtqvvxs8xg5cq/ulp";
    const DEMO: &str = "factory/kujira1ltvwg69sw3c5z99c6rr08hal7v0kdzfxz07yj5/demo";
    const KUJI: &str = "ukuji";

    fn pool_addr() -> PoolAddress {
        PoolAddress::SeparateAddresses {
            swap: Addr::unchecked(SWAP_CONTRACT),
            liquidity: Addr::unchecked(POOL_CONTRACT),
        }
    }

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
                pool_addr(),
                Asset::new(AssetInfo::native(DEMO), amount),
                AssetInfo::native(KUJI),
                Some(Decimal::from_str("0.2").unwrap()),
                Some(max_spread()),
            )
            .unwrap();

        expect_eq(
            vec![wasm_execute(
                SWAP_CONTRACT,
                &fin::ExecuteMsg::Swap {
                    offer_asset: Some(coin(amount, DEMO)),
                    belief_price: Some(Decimal256::from_str("0.2").unwrap()),
                    max_spread: Some(decimal2decimal256(max_spread()).unwrap()),
                    to: None,
                    callback: None,
                },
                coins(amount, DEMO),
            )
            .unwrap()
            .into()],
            msgs,
        )
        .unwrap();
    }

    #[test]
    fn provide_liquidity() {
        let amount_demo = 100_000u128;
        let amount_kuji = 50_000u128;
        let msgs = create_setup()
            .test_provide_liquidity(
                pool_addr(),
                vec![
                    Asset::new(AssetInfo::native(DEMO), amount_demo),
                    Asset::new(AssetInfo::native(KUJI), amount_kuji),
                ],
                Some(max_spread()),
            )
            .unwrap();

        expect_eq(
            vec![wasm_execute(
                POOL_CONTRACT,
                &bow::market_maker::ExecuteMsg::Deposit {
                    max_slippage: Some(max_spread()),
                    callback: None,
                },
                vec![coin(amount_demo, DEMO), coin(amount_kuji, KUJI)],
            )
            .unwrap()
            .into()],
            msgs,
        )
        .unwrap();
    }

    #[test]
    fn provide_liquidity_one_side() {
        let amount_demo = 100_000u128;
        let amount_kuji = 0u128;
        let msgs = create_setup()
            .test_provide_liquidity(
                pool_addr(),
                vec![
                    Asset::new(AssetInfo::native(DEMO), amount_demo),
                    Asset::new(AssetInfo::native(KUJI), amount_kuji),
                ],
                Some(max_spread()),
            )
            .unwrap();

        // There should be a swap before providing liquidity
        // We can't really test much further, because this unit test is querying mainnet liquidity pools
        expect_eq(
            wasm_execute(
                SWAP_CONTRACT,
                &fin::ExecuteMsg::Swap {
                    offer_asset: Some(coin(amount_demo / 2u128, DEMO)),
                    belief_price: None,
                    max_spread: Some(decimal2decimal256(max_spread()).unwrap()),
                    to: None,
                    callback: None,
                },
                coins(amount_demo / 2u128, DEMO),
            )
            .unwrap()
            .into(),
            msgs[0].clone(),
        )
        .unwrap();
    }

    #[test]
    fn provide_liquidity_symmetric() {
        let amount_demo = 100_000u128;
        let msgs = create_setup()
            .test_provide_liquidity_symmetric(
                pool_addr(),
                Asset::new(AssetInfo::native(DEMO), amount_demo),
                vec![AssetInfo::native(KUJI)],
            )
            .unwrap();

        assert_eq!(msgs.len(), 1);
        assert_eq!(get_wasm_addr(msgs[0].clone()), POOL_CONTRACT);

        let unwrapped_msg: bow::market_maker::ExecuteMsg = get_wasm_msg(msgs[0].clone());
        match unwrapped_msg {
            bow::market_maker::ExecuteMsg::Deposit {
                max_slippage,
                callback,
            } => {
                assert_eq!(max_slippage, None);
                assert_eq!(callback, None);
            }
            _ => panic!("Expected a provide liquidity variant"),
        }

        let funds = get_wasm_funds(msgs[0].clone());
        assert_eq!(funds.len(), 2);
        assert_eq!(funds[0], coin(amount_demo, DEMO),);
    }

    #[test]
    fn withdraw_liquidity() {
        let amount_lp = 100_000u128;
        let msgs = create_setup()
            .test_withdraw_liquidity(
                pool_addr(),
                Asset::new(AssetInfo::native(Addr::unchecked(LP_TOKEN)), amount_lp),
            )
            .unwrap();

        assert_eq!(
            msgs,
            vec![wasm_execute(
                POOL_CONTRACT,
                &bow::market_maker::ExecuteMsg::Withdraw { callback: None },
                coins(amount_lp, LP_TOKEN)
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
                pool_addr(),
                Asset::new(AssetInfo::native(DEMO), amount),
                AssetInfo::native(KUJI),
            )
            .unwrap();
    }
}
