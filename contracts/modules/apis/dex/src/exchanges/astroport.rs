use crate::{
    dex_trait::{Fee, FeeOnInput, Identify, Return, Spread},
    error::DexError,
    DEX,
};
use abstract_os::objects::PoolAddress;
use abstract_sdk::helpers::cosmwasm_std::wasm_smart_query;
use astroport::pair::{PoolResponse, SimulationResponse};
use cosmwasm_std::{
    to_binary, wasm_execute, Addr, Coin, CosmosMsg, Decimal, Deps, StdResult, WasmMsg,
};
use cw20::Cw20ExecuteMsg;
use cw_asset::{Asset, AssetInfo, AssetInfoBase};
pub const ASTROPORT: &str = "astroport";

// Source https://github.com/astroport-fi/astroport-core
pub struct Astroport {}

impl Identify for Astroport {
    fn name(&self) -> &'static str {
        ASTROPORT
    }
    fn over_ibc(&self) -> bool {
        false
    }
}

/// This structure describes a CW20 hook message.
#[cosmwasm_schema::cw_serde]
pub enum StubCw20HookMsg {
    /// Withdraw liquidity from the pool
    WithdrawLiquidity {},
}

impl DEX for Astroport {
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
                    msg: to_binary(&astroport::pair::Cw20HookMsg::Swap {
                        ask_asset_info: None,
                        belief_price,
                        max_spread,
                        to: None,
                    })?,
                },
                vec![],
            )?
            .into()],
            AssetInfo::Cw1155(..) => return Err(DexError::Cw1155Unsupported {}),
            _ => panic!("unsupported asset"),
        };
        Ok(swap_msg)
    }

    fn provide_liquidity(
        &self,
        _deps: Deps,
        pool_id: PoolAddress,
        offer_assets: Vec<Asset>,
        max_spread: Option<Decimal>,
    ) -> Result<Vec<CosmosMsg>, DexError> {
        let pair_address = pool_id.expect_contract()?;

        if offer_assets.len() > 2 {
            return Err(DexError::TooManyAssets(2));
        }

        let astroport_assets = offer_assets
            .iter()
            .map(cw_asset_to_astroport)
            .collect::<Result<Vec<_>, _>>()?;

        // execute msg
        let msg = astroport::pair::ExecuteMsg::ProvideLiquidity {
            assets: astroport_assets,
            slippage_tolerance: max_spread,
            auto_stake: Some(false),
            receiver: None,
        };

        // approval msgs for cw20 tokens (if present)
        let mut msgs = cw_approve_msgs(&offer_assets, &pair_address)?;
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
        let pair_config: PoolResponse = deps.querier.query(&wasm_smart_query(
            pair_address.to_string(),
            &astroport::pair::QueryMsg::Pool {},
        )?)?;
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
        #[cfg(not(feature = "testing"))]
        let hook_msg = StubCw20HookMsg::WithdrawLiquidity {};
        #[cfg(feature = "testing")]
        let hook_msg = astroport::pair::Cw20HookMsg::WithdrawLiquidity { assets: vec![] };

        let withdraw_msg = lp_token.send_msg(pair_address, to_binary(&hook_msg)?)?;
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
        } = deps.querier.query(&wasm_smart_query(
            pair_address.to_string(),
            &astroport::pair::QueryMsg::Simulation {
                offer_asset: cw_asset_to_astroport(&offer_asset)?,
                ask_asset_info: None,
            },
        )?)?;
        // commission paid in result asset
        Ok((return_amount, spread_amount, commission_amount, false))
    }
}

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
        _ => Err(DexError::Cw1155Unsupported {}),
    }
}

fn cw_approve_msgs(assets: &[Asset], spender: &Addr) -> StdResult<Vec<CosmosMsg>> {
    let mut msgs = vec![];
    for asset in assets {
        if let AssetInfo::Cw20(addr) = &asset.info {
            let msg = cw20_junoswap::Cw20ExecuteMsg::IncreaseAllowance {
                spender: spender.to_string(),
                amount: asset.amount,
                expires: None,
            };
            msgs.push(CosmosMsg::Wasm(WasmMsg::Execute {
                contract_addr: addr.to_string(),
                msg: to_binary(&msg)?,
                funds: vec![],
            }))
        }
    }
    Ok(msgs)
}

fn coins_in_assets(assets: &[Asset]) -> Vec<Coin> {
    let mut coins = vec![];
    for asset in assets {
        if let AssetInfo::Native(denom) = &asset.info {
            coins.push(Coin::new(asset.amount.u128(), denom.clone()));
        }
    }
    coins
}
