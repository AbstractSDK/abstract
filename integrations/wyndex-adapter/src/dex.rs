use abstract_dex_standard::Identify;

use crate::{AVAILABLE_CHAINS, WYNDEX};
// Source https://github.com/cosmorama/wynddex/tree/v1.0.0
#[derive(Default)]
pub struct WynDex {}

impl Identify for WynDex {
    fn name(&self) -> &'static str {
        WYNDEX
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
    cosmwasm_std::{to_json_binary, wasm_execute, CosmosMsg, Decimal, Deps, Uint128},
    cw20::Cw20ExecuteMsg,
    cw_asset::{Asset, AssetInfo, AssetInfoBase},
    wyndex::pair::*,
};

#[cfg(feature = "full_integration")]
impl DexCommand for WynDex {
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
            AssetInfo::Native(_) => Ok(vec![wasm_execute(
                pair_address.to_string(),
                &ExecuteMsg::Swap {
                    offer_asset: cw_asset_to_wyndex(&offer_asset)?,
                    ask_asset_info: None,
                    belief_price,
                    max_spread,
                    to: None,
                    referral_address: None,
                    referral_commission: None,
                },
                vec![offer_asset.clone().try_into()?],
            )?
            .into()]),
            AssetInfo::Cw20(addr) => Ok(vec![wasm_execute(
                addr.to_string(),
                &Cw20ExecuteMsg::Send {
                    contract: pair_address.to_string(),
                    amount: offer_asset.amount,
                    msg: to_json_binary(&Cw20HookMsg::Swap {
                        ask_asset_info: None,
                        belief_price,
                        max_spread,
                        to: None,
                        referral_address: None,
                        referral_commission: None,
                    })?,
                },
                vec![],
            )?
            .into()]),
            _ => Err(DexError::UnsupportedAssetType(offer_asset.info.to_string())),
        }?;
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

        let wyndex_assets = offer_assets
            .iter()
            .map(cw_asset_to_wyndex)
            .collect::<Result<Vec<_>, _>>()?;

        // execute msg
        let msg = ExecuteMsg::ProvideLiquidity {
            assets: wyndex_assets,
            slippage_tolerance: max_spread,
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

    fn withdraw_liquidity(
        &self,
        _deps: Deps,
        pool_id: PoolAddress,
        lp_token: Asset,
    ) -> Result<Vec<CosmosMsg>, DexError> {
        let pair_address = pool_id.expect_contract()?;
        let hook_msg = Cw20HookMsg::WithdrawLiquidity { assets: vec![] };
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
            ..
        } = deps.querier.query_wasm_smart(
            pair_address.to_string(),
            &QueryMsg::Simulation {
                referral: false,
                referral_commission: None,
                offer_asset: cw_asset_to_wyndex(&offer_asset)?,
                ask_asset_info: None,
            },
        )?;
        // commission paid in result asset
        Ok((return_amount, spread_amount, commission_amount, false))
    }
}

#[cfg(feature = "full_integration")]
fn cw_asset_to_wyndex(asset: &Asset) -> Result<wyndex::asset::Asset, DexError> {
    match &asset.info {
        AssetInfoBase::Native(denom) => Ok(wyndex::asset::Asset {
            amount: asset.amount,
            info: wyndex::asset::AssetInfo::Native(denom.clone()),
        }),
        AssetInfoBase::Cw20(contract_addr) => Ok(wyndex::asset::Asset {
            amount: asset.amount,
            info: wyndex::asset::AssetInfo::Token(contract_addr.to_string()),
        }),
        _ => Err(DexError::UnsupportedAssetType(asset.to_string())),
    }
}
