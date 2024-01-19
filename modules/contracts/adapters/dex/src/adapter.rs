use crate::state::SWAP_FEE;
use abstract_core::objects::AnsEntryConvertor;
use abstract_core::objects::{DexAssetPairing, PoolReference};

use abstract_core::version_control::AccountBase;
use abstract_dex_standard::msg::{DexAction, OfferAsset};
use abstract_dex_standard::DexError;
use abstract_sdk::core::objects::AnsAsset;
use abstract_sdk::core::objects::AssetEntry;
use abstract_sdk::cw_helpers::Chargeable;
use abstract_sdk::features::{AbstractNameService, AbstractRegistryAccess};
use abstract_sdk::Execution;
use cosmwasm_std::{CosmosMsg, Decimal, Deps, StdError};

use cw_asset::Asset;

use abstract_dex_standard::DexCommand;

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
        target_account: AccountBase,
        action: DexAction,
        mut exchange: Box<dyn DexCommand>,
    ) -> Result<(Vec<CosmosMsg>, ReplyId), DexError> {
        Ok(match action {
            DexAction::ProvideLiquidity { assets, max_spread } => {
                if assets.len() < 2 {
                    return Err(DexError::TooFewAssets {});
                }
                (
                    self.resolve_provide_liquidity(
                        deps,
                        target_account,
                        assets,
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
                        target_account,
                        offer_asset,
                        paired_assets,
                        exchange.as_mut(),
                    )?,
                    PROVIDE_LIQUIDITY_SYM,
                )
            }
            DexAction::WithdrawLiquidity { lp_token, amount } => (
                self.resolve_withdraw_liquidity(
                    deps,
                    target_account,
                    AnsAsset::new(lp_token, amount),
                    exchange.as_mut(),
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
                    deps,
                    target_account,
                    offer_asset,
                    ask_asset,
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
        target_account: AccountBase,
        offer_asset: OfferAsset,
        mut ask_asset: AssetEntry,
        exchange: &mut dyn DexCommand,
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

        let PoolReference {
            unique_id,
            pool_address,
        } = exchange.pool_reference(deps, ans.host(), (offer_asset.clone(), ask_asset))?;
        let mut offer_asset: Asset = Asset::new(offer_asset_info, offer_amount);
        // account for fee
        let fee = SWAP_FEE.load(deps.storage)?;
        let fee_msg = offer_asset.charge_usage_fee(fee)?;

        exchange.fetch_data(
            deps,
            target_account,
            self.abstract_registry(deps)?,
            self.ans_host(deps)?,
            unique_id,
        )?;
        let mut swap_msgs = exchange.swap(
            deps,
            pool_address,
            offer_asset,
            ask_asset_info,
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
        target_account: AccountBase,
        offer_assets: Vec<OfferAsset>,
        exchange: &mut dyn DexCommand,
        max_spread: Option<Decimal>,
    ) -> Result<Vec<CosmosMsg>, DexError> {
        let ans = self.name_service(deps);
        let assets = ans.query(&offer_assets)?;

        let mut pair_assets = offer_assets
            .into_iter()
            .map(|a| a.name)
            .take(2)
            .collect::<Vec<AssetEntry>>();

        let PoolReference {
            unique_id,
            pool_address,
        } = exchange.pool_reference(
            deps,
            ans.host(),
            (pair_assets.swap_remove(0), pair_assets.swap_remove(0)),
        )?;
        exchange.fetch_data(
            deps,
            target_account,
            self.abstract_registry(deps)?,
            self.ans_host(deps)?,
            unique_id,
        )?;
        exchange.provide_liquidity(deps, pool_address, assets, max_spread)
    }

    fn resolve_provide_liquidity_symmetric(
        &self,
        deps: Deps,
        target_account: AccountBase,
        offer_asset: OfferAsset,
        mut paired_assets: Vec<AssetEntry>,
        exchange: &mut dyn DexCommand,
    ) -> Result<Vec<CosmosMsg>, DexError> {
        let ans = self.name_service(deps);
        let paired_asset_infos = ans.query(&paired_assets)?;
        let PoolReference {
            pool_address,
            unique_id,
        } = exchange.pool_reference(
            deps,
            ans.host(),
            (paired_assets.swap_remove(0), offer_asset.name.clone()),
        )?;
        let offer_asset = ans.query(&offer_asset)?;
        exchange.fetch_data(
            deps,
            target_account,
            self.abstract_registry(deps)?,
            self.ans_host(deps)?,
            unique_id,
        )?;
        exchange.provide_liquidity_symmetric(deps, pool_address, offer_asset, paired_asset_infos)
    }

    /// @todo
    fn resolve_withdraw_liquidity(
        &self,
        deps: Deps,
        target_account: AccountBase,
        lp_token: OfferAsset,
        exchange: &mut dyn DexCommand,
    ) -> Result<Vec<CosmosMsg>, DexError> {
        let ans = self.name_service(deps);

        let lp_asset = ans.query(&lp_token)?;

        let lp_pairing: DexAssetPairing =
            AnsEntryConvertor::new(AnsEntryConvertor::new(lp_token.name).lp_token()?)
                .dex_asset_pairing()?;

        let mut pool_ids = ans.query(&lp_pairing)?;
        // TODO: when resolving if there are more than one, get the metadata and choose the one matching the assets
        if pool_ids.len() != 1 {
            return Err(StdError::generic_err(format!(
                "There are {} pairings for the given LP token",
                pool_ids.len()
            ))
            .into());
        }

        let PoolReference {
            pool_address,
            unique_id,
        } = pool_ids.pop().unwrap();
        exchange.fetch_data(
            deps,
            target_account,
            self.abstract_registry(deps)?,
            self.ans_host(deps)?,
            unique_id,
        )?;
        exchange.withdraw_liquidity(deps, pool_address, lp_asset)
    }
}
