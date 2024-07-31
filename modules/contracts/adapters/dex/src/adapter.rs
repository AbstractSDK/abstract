use abstract_adapter::sdk::{
    cw_helpers::Chargeable,
    features::{AbstractNameService, AbstractRegistryAccess},
    Execution,
};
use abstract_adapter::std::objects::pool_id::PoolAddressBase;
use abstract_dex_standard::{msg::SwapNode, action::DexAction, DexCommand, DexError};
use cosmwasm_std::{Addr, CosmosMsg, Decimal, Deps};
use cw_asset::{AssetBase, AssetInfoBase};

use crate::state::DEX_FEES;

pub const PROVIDE_LIQUIDITY: u64 = 7542;
pub const PROVIDE_LIQUIDITY_SYM: u64 = 7543;
pub const WITHDRAW_LIQUIDITY: u64 = 7546;
pub const SWAP: u64 = 7544;
pub const SWAP_ROUTE: u64 = 7545;

impl<T> DexAdapter for T where T: AbstractNameService + Execution + AbstractRegistryAccess {}

pub(crate) type ReplyId = u64;

pub trait DexAdapter: AbstractNameService + AbstractRegistryAccess + Execution {
    /// resolve the provided dex action on a local dex
    fn resolve_dex_action(
        &self,
        deps: Deps,
        sender: Addr,
        action: DexAction,
        mut exchange: Box<dyn DexCommand>,
    ) -> Result<(Vec<CosmosMsg>, ReplyId), DexError> {
        Ok(match action {
            DexAction::ProvideLiquidity {
                pool,
                assets,
                max_spread,
            } => {
                if assets.len() < 2 {
                    return Err(DexError::TooFewAssets {});
                }
                (
                    self.resolve_provide_liquidity(
                        deps,
                        sender,
                        assets,
                        pool,
                        exchange.as_mut(),
                        max_spread,
                    )?,
                    PROVIDE_LIQUIDITY,
                )
            }
            DexAction::WithdrawLiquidity { pool, lp_token } => (
                self.resolve_withdraw_liquidity(deps, sender, lp_token, pool, exchange.as_mut())?,
                WITHDRAW_LIQUIDITY,
            ),
            DexAction::Swap {
                pool,
                offer_asset,
                ask_asset,
                max_spread,
                belief_price,
            } => (
                self.resolve_swap(
                    deps,
                    sender,
                    offer_asset,
                    ask_asset,
                    pool,
                    exchange.as_mut(),
                    max_spread,
                    belief_price,
                )?,
                SWAP,
            ),
            DexRawAction::RouteSwap {
                route,
                offer_asset,
                max_spread,
                belief_price,
            } => (
                self.resolve_route_swap(
                    deps,
                    sender,
                    offer_asset,
                    route,
                    exchange.as_mut(),
                    max_spread,
                    belief_price,
                )?,
                SWAP_ROUTE,
            ),
        })
    }

    #[allow(clippy::too_many_arguments)]
    fn resolve_swap(
        &self,
        deps: Deps,
        sender: Addr,
        offer_asset: AssetBase<String>,
        ask_asset: AssetInfoBase<String>,
        pool: PoolAddressBase<String>,
        exchange: &mut dyn DexCommand,
        max_spread: Option<Decimal>,
        belief_price: Option<Decimal>,
    ) -> Result<Vec<CosmosMsg>, DexError> {
        let pool_address = pool.check(deps.api)?;
        let mut offer_asset = offer_asset.check(deps.api, None)?;
        let ask_asset = ask_asset.check(deps.api, None)?;

        // account for fee
        let dex_fees = DEX_FEES.load(deps.storage)?;
        let usage_fee = dex_fees.swap_usage_fee()?;
        let fee_msg = offer_asset.charge_usage_fee(usage_fee)?;

        exchange.fetch_data(
            deps,
            sender,
            self.abstract_registry(deps)?,
            self.ans_host(deps)?,
        )?;
        let mut swap_msgs = exchange.swap(
            deps,
            pool_address,
            offer_asset,
            ask_asset,
            belief_price,
            max_spread,
        )?;
        // insert fee msg
        if let Some(f) = fee_msg {
            swap_msgs.push(f)
        }

        Ok(swap_msgs)
    }

    #[allow(clippy::too_many_arguments)]
    fn resolve_route_swap(
        &self,
        deps: Deps,
        sender: Addr,
        offer_asset: AssetBase<String>,
        swap_route: Vec<SwapNode<String>>,
        exchange: &mut dyn DexCommand,
        max_spread: Option<Decimal>,
        belief_price: Option<Decimal>,
    ) -> Result<Vec<CosmosMsg>, DexError> {
        let mut offer_asset = offer_asset.check(deps.api, None)?;
        let swap_route = swap_route
            .into_iter()
            .map(|node| node.check(deps.api))
            .collect::<abstract_adapter::std::AbstractResult<_>>()?;

        // account for fee
        let dex_fees = DEX_FEES.load(deps.storage)?;
        let usage_fee = dex_fees.swap_usage_fee()?;
        let fee_msg = offer_asset.charge_usage_fee(usage_fee)?;

        exchange.fetch_data(
            deps,
            sender,
            self.abstract_registry(deps)?,
            self.ans_host(deps)?,
        )?;
        let mut swap_msgs =
            exchange.swap_route(deps, swap_route, offer_asset, belief_price, max_spread)?;
        // insert fee msg
        if let Some(f) = fee_msg {
            swap_msgs.push(f)
        }

        Ok(swap_msgs)
    }

    fn resolve_provide_liquidity(
        &self,
        deps: Deps,
        sender: Addr,
        offer_assets: Vec<AssetBase<String>>,
        pool: PoolAddressBase<String>,
        exchange: &mut dyn DexCommand,
        max_spread: Option<Decimal>,
    ) -> Result<Vec<CosmosMsg>, DexError> {
        let pool_address = pool.check(deps.api)?;
        let offer_assets = offer_assets
            .into_iter()
            .map(|o| o.check(deps.api, None))
            .collect::<Result<_, _>>()?;

        exchange.fetch_data(
            deps,
            sender,
            self.abstract_registry(deps)?,
            self.ans_host(deps)?,
        )?;
        exchange.provide_liquidity(deps, pool_address, offer_assets, max_spread)
    }

    /// @todo
    fn resolve_withdraw_liquidity(
        &self,
        deps: Deps,
        sender: Addr,
        lp_token: AssetBase<String>,
        pool: PoolAddressBase<String>,
        exchange: &mut dyn DexCommand,
    ) -> Result<Vec<CosmosMsg>, DexError> {
        let pool_address = pool.check(deps.api)?;
        let lp_token = lp_token.check(deps.api, None)?;

        exchange.fetch_data(
            deps,
            sender,
            self.abstract_registry(deps)?,
            self.ans_host(deps)?,
        )?;
        exchange.withdraw_liquidity(deps, pool_address, lp_token)
    }
}
