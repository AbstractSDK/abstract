use abstract_core::objects::{AnsEntryConvertor, PoolAddress};
use abstract_dex_standard::{
    msg::{AskAsset, DexAction, OfferAsset},
    DexCommand, DexError,
};
use abstract_sdk::{
    core::objects::AnsAsset,
    cw_helpers::Chargeable,
    features::{AbstractNameService, AbstractRegistryAccess},
    Execution,
};
use cosmwasm_std::{Addr, CosmosMsg, Decimal, Deps, StdError};
use cw_asset::{Asset, AssetInfo};

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
        action: DexAction,
        mut exchange: Box<dyn DexCommand>,
        pool: Option<PoolAddress>,
    ) -> Result<(Vec<CosmosMsg>, ReplyId), DexError> {
        Ok(match action {
            DexAction::ProvideLiquidity { assets, max_spread } => {
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
            DexAction::ProvideLiquiditySymmetric {
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
            DexAction::WithdrawLiquidity { lp_token } => (
                self.resolve_withdraw_liquidity(deps, sender, lp_token, pool, exchange.as_mut())?,
                WITHDRAW_LIQUIDITY,
            ),
            DexAction::Swap {
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
        offer_asset: OfferAsset,
        ask_asset: AskAsset,
        pool: Option<PoolAddress>,
        exchange: &mut dyn DexCommand,
        max_spread: Option<Decimal>,
        belief_price: Option<Decimal>,
    ) -> Result<Vec<CosmosMsg>, DexError> {
        // We resolve the offer asset if needed
        let mut offer_cw_asset = self._get_offer_asset(deps, &offer_asset)?;
        let ask_cw_asset = self._get_ask_asset(deps, &ask_asset)?;
        // We resolve the ask asset if needed

        // If the pool is not specified, we query for it
        let pool_address = self._get_pool(deps, exchange, pool, &offer_asset.info(), &ask_asset)?;

        // account for fee
        let dex_fees = DEX_FEES.load(deps.storage)?;
        let usage_fee = dex_fees.swap_usage_fee()?;
        let fee_msg = offer_cw_asset.charge_usage_fee(usage_fee)?;

        exchange.fetch_data(
            deps,
            sender,
            self.abstract_registry(deps)?,
            self.ans_host(deps)?,
        )?;
        let mut swap_msgs = exchange.swap(
            deps,
            pool_address,
            offer_cw_asset,
            ask_cw_asset,
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
        offer_assets: Vec<OfferAsset>,
        pool: Option<PoolAddress>,
        exchange: &mut dyn DexCommand,
        max_spread: Option<Decimal>,
    ) -> Result<Vec<CosmosMsg>, DexError> {
        let assets = offer_assets
            .iter()
            .map(|offer| self._get_offer_asset(deps, offer))
            .collect::<Result<Vec<_>, _>>()?;

        let pair_assets = offer_assets
            .into_iter()
            .map(|a| a.info())
            .take(2)
            .collect::<Vec<_>>();

        let pool_address =
            self._get_pool(deps, exchange, pool, &pair_assets[0], &pair_assets[1])?;

        exchange.fetch_data(
            deps,
            sender,
            self.abstract_registry(deps)?,
            self.ans_host(deps)?,
        )?;
        exchange.provide_liquidity(deps, pool_address, assets, max_spread)
    }

    fn resolve_provide_liquidity_symmetric(
        &self,
        deps: Deps,
        sender: Addr,
        pool: Option<PoolAddress>,
        offer_asset: OfferAsset,
        mut paired_assets: Vec<AskAsset>,
        exchange: &mut dyn DexCommand,
    ) -> Result<Vec<CosmosMsg>, DexError> {
        let paired_asset_infos = paired_assets
            .iter()
            .map(|ask| self._get_ask_asset(deps, ask))
            .collect::<Result<Vec<_>, _>>()?;

        let pool_address = self._get_pool(
            deps,
            exchange,
            pool,
            &paired_assets.swap_remove(0),
            &offer_asset.info(),
        )?;

        let offer_asset = self._get_offer_asset(deps, &offer_asset)?;
        exchange.fetch_data(
            deps,
            sender,
            self.abstract_registry(deps)?,
            self.ans_host(deps)?,
        )?;
        exchange.provide_liquidity_symmetric(deps, pool_address, offer_asset, paired_asset_infos)
    }

    /// @todo
    fn resolve_withdraw_liquidity(
        &self,
        deps: Deps,
        sender: Addr,
        lp_token: OfferAsset,
        pool: Option<PoolAddress>,
        exchange: &mut dyn DexCommand,
    ) -> Result<Vec<CosmosMsg>, DexError> {
        let ans = self.name_service(deps);

        let lp_asset = self._get_offer_asset(deps, &lp_token)?;

        let pool_address = match lp_token.info() {
            AskAsset::Ans(ans_asset) => {
                let pairing = AnsEntryConvertor::new(AnsEntryConvertor::new(ans_asset).lp_token()?)
                    .dex_asset_pairing()?;
                let mut pool_ids = ans.query(&pairing)?;
                // TODO: when resolving if there are more than one, get the metadata and choose the one matching the assets
                if pool_ids.len() != 1 {
                    return Err(StdError::generic_err(format!(
                        "There are {} pairings for the given LP token",
                        pool_ids.len()
                    ))
                    .into());
                }

                pool_ids.pop().unwrap().pool_address
            }
            AskAsset::Raw(_raw) => {
                // If a raw asset is provided, you also need to provide the pool Address with it
                pool.ok_or(DexError::PoolAddressEmpty)?
            }
        };

        exchange.fetch_data(
            deps,
            sender,
            self.abstract_registry(deps)?,
            self.ans_host(deps)?,
        )?;
        exchange.withdraw_liquidity(deps, pool_address, lp_asset)
    }

    fn _get_offer_asset(&self, deps: Deps, offer_asset: &OfferAsset) -> Result<Asset, DexError> {
        let ans = self.name_service(deps);

        Ok(match offer_asset {
            OfferAsset::Raw(offer_asset) => offer_asset.clone(),
            OfferAsset::Ans(ans_asset) => {
                let AnsAsset {
                    name: mut offer_asset,
                    amount: offer_amount,
                } = ans_asset.clone();
                offer_asset.format();

                let offer_asset_info: cw_asset::AssetInfoBase<Addr> = ans.query(&offer_asset)?;

                Asset::new(offer_asset_info, offer_amount)
            }
        })
    }
    fn _get_ask_asset(&self, deps: Deps, ask_asset: &AskAsset) -> Result<AssetInfo, DexError> {
        let ans = self.name_service(deps);

        Ok(match ask_asset.clone() {
            AskAsset::Raw(ask_asset) => ask_asset,
            AskAsset::Ans(mut ans_asset) => {
                ans_asset.format();
                ans.query(&ans_asset)?
            }
        })
    }

    fn _get_pool(
        &self,
        deps: Deps,
        exchange: &dyn DexCommand<DexError>,
        pool: Option<PoolAddress>,
        asset1: &AskAsset,
        asset2: &AskAsset,
    ) -> Result<PoolAddress, DexError> {
        let ans = self.name_service(deps);

        Ok(match pool {
            Some(pool_address) => pool_address,
            None => {
                let ans_asset_1 = match asset1 {
                    AskAsset::Raw(raw_asset) => ans.query(raw_asset)?,
                    AskAsset::Ans(ans_asset) => ans_asset.clone(),
                };
                let ans_asset_2 = match asset2 {
                    AskAsset::Raw(raw_asset) => ans.query(raw_asset)?,
                    AskAsset::Ans(ans_asset) => ans_asset.clone(),
                };

                exchange
                    .pool_reference(deps, ans.host(), (ans_asset_1, ans_asset_2))?
                    .pool_address
            }
        })
    }
}
