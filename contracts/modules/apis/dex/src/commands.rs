use abstract_os::objects::{DexAssetPairing, PoolReference};
use abstract_sdk::base::features::AbstractNameService;
use abstract_sdk::os::objects::AnsAsset;
use abstract_sdk::Execution;
use cosmwasm_std::{CosmosMsg, Decimal, Deps, DepsMut, ReplyOn, StdError, SubMsg};
use cw_asset::Asset;

use crate::{error::DexError, DEX};
use abstract_sdk::os::dex::AskAsset;
use abstract_sdk::os::{
    dex::{DexAction, OfferAsset, SwapRouter},
    objects::AssetEntry,
};

pub const PROVIDE_LIQUIDITY: u64 = 7542;
pub const PROVIDE_LIQUIDITY_SYM: u64 = 7543;
pub const WITHDRAW_LIQUIDITY: u64 = 7546;
pub const SWAP: u64 = 7544;
pub const CUSTOM_SWAP: u64 = 7545;

impl<T> LocalDex for T where T: AbstractNameService + Execution {}

pub trait LocalDex: AbstractNameService + Execution {
    /// resolve the provided dex action on a local dex
    fn resolve_dex_action(
        &self,
        deps: DepsMut,
        action: DexAction,
        exchange: &dyn DEX,
        with_reply: bool,
    ) -> Result<SubMsg, DexError> {
        let (msgs, reply_id) = match action {
            DexAction::ProvideLiquidity { assets, max_spread } => {
                if assets.len() < 2 {
                    return Err(DexError::TooFewAssets {});
                }
                (
                    self.resolve_provide_liquidity(deps.as_ref(), assets, exchange, max_spread)?,
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
                        deps.as_ref(),
                        offer_asset,
                        paired_assets,
                        exchange,
                    )?,
                    PROVIDE_LIQUIDITY_SYM,
                )
            }
            DexAction::WithdrawLiquidity { lp_token, amount } => (
                self.resolve_withdraw_liquidity(
                    deps.as_ref(),
                    AnsAsset::new(lp_token, amount),
                    exchange,
                )?,
                WITHDRAW_LIQUIDITY,
            ),
            DexAction::Swap {
                offer_asset,
                ask_asset,
                max_spread,
                belief_price,
            } => (
                self.resolve_swap(
                    deps.as_ref(),
                    offer_asset,
                    ask_asset,
                    exchange,
                    max_spread,
                    belief_price,
                )?,
                SWAP,
            ),
            DexAction::CustomSwap {
                offer_assets,
                ask_assets,
                max_spread,
                router,
            } => (
                self.resolve_custom_swap(
                    deps.as_ref(),
                    offer_assets,
                    ask_assets,
                    exchange,
                    max_spread,
                    router,
                )?,
                CUSTOM_SWAP,
            ),
        };
        if with_reply {
            self.executor(deps.as_ref())
                .execute_with_reply(msgs, ReplyOn::Success, reply_id)
        } else {
            self.executor(deps.as_ref()).execute(msgs).map(SubMsg::new)
        }
        .map_err(Into::into)
    }

    #[allow(clippy::too_many_arguments)]
    fn resolve_swap(
        &self,
        deps: Deps,
        offer_asset: OfferAsset,
        mut ask_asset: AssetEntry,
        exchange: &dyn DEX,
        max_spread: Option<Decimal>,
        belief_price: Option<Decimal>,
    ) -> Result<Vec<CosmosMsg>, DexError> {
        let AnsAsset {
            name: mut offer_asset,
            amount: offer_amount,
        } = offer_asset;
        offer_asset.format();
        ask_asset.format();

        let ans = self.name_service(deps);
        let offer_asset_info = ans.query(&offer_asset)?;
        let ask_asset_info = ans.query(&ask_asset)?;

        let pair_address =
            exchange.pair_address(deps, ans.host(), (offer_asset.clone(), ask_asset))?;
        let offer_asset: Asset = Asset::new(offer_asset_info, offer_amount);

        exchange.swap(
            deps,
            pair_address,
            offer_asset,
            ask_asset_info,
            belief_price,
            max_spread,
        )
    }

    #[allow(clippy::too_many_arguments)]
    fn resolve_custom_swap(
        &self,
        _deps: Deps,
        _offer_assets: Vec<OfferAsset>,
        _ask_assets: Vec<AskAsset>,
        _exchange: &dyn DEX,
        _max_spread: Option<Decimal>,
        _router: Option<SwapRouter>,
    ) -> Result<Vec<CosmosMsg>, DexError> {
        todo!()

        // let ans_host = api.ans(deps);
        //
        // // Resolve the asset information
        // let mut offer_asset_infos: Vec<AssetInfo> =
        //     exchange.resolve_assets(deps, &api, offer_assets.into_iter().unzip().0)?;
        // let mut ask_asset_infos: Vec<AssetInfo> =
        //     exchange.resolve_assets(deps, &api, ask_assets.into_iter().unzip().0)?;
        //
        // let offer_assets: Vec<Asset> = offer_assets
        //     .into_iter()
        //     .zip(offer_asset_infos)
        //     .map(|(asset, info)| Asset::new(info, asset.1))
        //     .collect();
        // let ask_assets: Vec<Asset> = ask_assets
        //     .into_iter()
        //     .zip(ask_asset_infos)
        //     .map(|(asset, info)| Asset::new(info, asset.1))
        //     .collect();
        //
        // exchange.custom_swap(deps, offer_assets, ask_assets, max_spread)
    }

    fn resolve_provide_liquidity(
        &self,
        deps: Deps,
        offer_assets: Vec<OfferAsset>,
        exchange: &dyn DEX,
        max_spread: Option<Decimal>,
    ) -> Result<Vec<CosmosMsg>, DexError> {
        let ans = self.name_service(deps);
        let assets = ans.query(&offer_assets)?;

        let mut pair_assets = offer_assets
            .into_iter()
            .map(|a| a.name)
            .take(2)
            .collect::<Vec<AssetEntry>>();

        let pair_address = exchange.pair_address(
            deps,
            ans.host(),
            (pair_assets.swap_remove(0), pair_assets.swap_remove(1)),
        )?;
        exchange.provide_liquidity(deps, pair_address, assets, max_spread)
    }

    fn resolve_provide_liquidity_symmetric(
        &self,
        deps: Deps,
        offer_asset: OfferAsset,
        mut paired_assets: Vec<AssetEntry>,
        exchange: &dyn DEX,
    ) -> Result<Vec<CosmosMsg>, DexError> {
        let ans = self.name_service(deps);
        let paired_asset_infos = ans.query(&paired_assets)?;
        let pair_address = exchange.pair_address(
            deps,
            ans.host(),
            (paired_assets.swap_remove(0), paired_assets.swap_remove(1)),
        )?;
        let offer_asset = ans.query(&offer_asset)?;
        exchange.provide_liquidity_symmetric(deps, pair_address, offer_asset, paired_asset_infos)
    }

    /// @todo
    fn resolve_withdraw_liquidity(
        &self,
        deps: Deps,
        lp_token: OfferAsset,
        exchange: &dyn DEX,
    ) -> Result<Vec<CosmosMsg>, DexError> {
        let ans = self.name_service(deps);

        let lp_asset = ans.query(&lp_token)?;

        let lp_pairing: DexAssetPairing = lp_token.name.try_into()?;

        let mut pool_ids = ans.query(&lp_pairing)?;
        // TODO: when resolving if there are more than one, get the metadata and choose the one matching the assets
        if pool_ids.len() != 1 {
            return Err(StdError::generic_err(format!(
                "There are {} pairings for the given LP token",
                pool_ids.len()
            ))
            .into());
        }

        let PoolReference { pool_address, .. } = pool_ids.pop().unwrap();
        exchange.withdraw_liquidity(deps, pool_address, lp_asset)
    }
}
