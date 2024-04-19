use abstract_adapter::abstract_std::objects::pool_id::PoolAddressBase;
use abstract_adapter::sdk::{
    cw_helpers::Chargeable,
    features::{AbstractNameService, AbstractRegistryAccess},
    Execution,
};
use abstract_dex_standard::{raw_action::DexRawAction, DexCommand, DexError};
use cosmwasm_std::{Addr, CosmosMsg, Decimal, Deps};
use cw_asset::{AssetBase, AssetInfoBase};

use crate::state::DEX_FEES;

pub const PROVIDE_LIQUIDITY: u64 = 7542;
pub const PROVIDE_LIQUIDITY_SYM: u64 = 7543;
pub const WITHDRAW_LIQUIDITY: u64 = 7546;
pub const SWAP: u64 = 7544;
pub const CUSTOM_SWAP: u64 = 7545;

impl<T> DexAdapter for T where T: AbstractNameService + Execution + AbstractRegistryAccess {}

pub(crate) type ReplyId = u64;

pub trait DexAdapter: AbstractNameService + AbstractRegistryAccess + Execution {
    /// resolve the provided dex action on a local dex
    fn resolve_dex_action(
        &self,
        deps: Deps,
        sender: Addr,
        action: DexRawAction,
        mut exchange: Box<dyn DexCommand>,
    ) -> Result<(Vec<CosmosMsg>, ReplyId), DexError> {
        Ok(match action {
            DexRawAction::ProvideLiquidity {
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
            DexRawAction::ProvideLiquiditySymmetric {
                pool,
                offer_asset,
                paired_assets,
            } => {
                if paired_assets.is_empty() {
                    return Err(DexError::TooFewAssets {});
                }
                (
                    self.resolve_provide_liquidity_symmetric(
                        deps,
                        sender,
                        pool,
                        offer_asset,
                        paired_assets,
                        exchange.as_mut(),
                    )?,
                    PROVIDE_LIQUIDITY_SYM,
                )
            }
            DexRawAction::WithdrawLiquidity { pool, lp_token } => (
                self.resolve_withdraw_liquidity(deps, sender, lp_token, pool, exchange.as_mut())?,
                WITHDRAW_LIQUIDITY,
            ),
            DexRawAction::Swap {
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

    fn resolve_provide_liquidity_symmetric(
        &self,
        deps: Deps,
        sender: Addr,
        pool: PoolAddressBase<String>,
        offer_asset: AssetBase<String>,
        paired_assets: Vec<AssetInfoBase<String>>,
        exchange: &mut dyn DexCommand,
    ) -> Result<Vec<CosmosMsg>, DexError> {
        let pool_address = pool.check(deps.api)?;
        let paired_assets = paired_assets
            .into_iter()
            .map(|o| o.check(deps.api, None))
            .collect::<Result<_, _>>()?;
        let offer_asset = offer_asset.check(deps.api, None)?;

        exchange.fetch_data(
            deps,
            sender,
            self.abstract_registry(deps)?,
            self.ans_host(deps)?,
        )?;
        exchange.provide_liquidity_symmetric(deps, pool_address, offer_asset, paired_assets)
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
