#![warn(missing_docs)]
//! # Dex Adapter ANS Action Definition
//!
use abstract_sdk::{feature_objects::AnsHost, Resolve};
use abstract_std::objects::{
    ans_host::AnsHostError, AnsAsset, AnsEntryConvertor, AssetEntry, DexAssetPairing, PoolAddress,
    PoolReference,
};
use cosmwasm_std::{Decimal, StdError};
use cw_asset::Asset;

use crate::{msg::DexName, raw_action::DexRawAction};

/// Possible actions to perform on the DEX
#[cosmwasm_schema::cw_serde]
pub enum DexAnsAction {
    /// Provide arbitrary liquidity
    ProvideLiquidity {
        // support complex pool types
        /// Assets to add
        assets: Vec<AnsAsset>,
        /// Max spread to accept, is a percentage represented as a decimal.
        max_spread: Option<Decimal>,
    },
    /// Withdraw liquidity from a pool
    WithdrawLiquidity {
        /// The asset LP token that is provided.
        lp_token: AnsAsset,
    },
    /// Standard swap between one asset to another
    Swap {
        /// The asset to offer
        offer_asset: AnsAsset,
        /// The asset to receive
        ask_asset: AssetEntry,
        /// The percentage of spread compared to pre-swap price or belief price (if provided)
        max_spread: Option<Decimal>,
        /// The belief price when submitting the transaction.
        belief_price: Option<Decimal>,
    },
}
/// Structure created to be able to resolve an action using ANS
pub struct WholeDexAction(pub DexName, pub DexAnsAction);

/// Returns the first pool address to be able to swap given assets on the given dex
pub fn pool_address(
    dex: DexName,
    assets: (AssetEntry, AssetEntry),
    querier: &cosmwasm_std::QuerierWrapper,
    ans_host: &AnsHost,
) -> abstract_std::objects::ans_host::AnsHostResult<PoolAddress> {
    let dex_pair = DexAssetPairing::new(assets.0, assets.1, &dex);
    let mut pool_ref = ans_host.query_asset_pairing(querier, &dex_pair)?;
    // Currently takes the first pool found, but should be changed to take the best pool
    let found: PoolReference = pool_ref.pop().ok_or(AnsHostError::DexPairingNotFound {
        pairing: dex_pair,
        ans_host: ans_host.address.clone(),
    })?;
    Ok(found.pool_address)
}

impl Resolve for WholeDexAction {
    type Output = DexRawAction;

    fn resolve(
        &self,
        querier: &cosmwasm_std::QuerierWrapper,
        ans_host: &abstract_sdk::feature_objects::AnsHost,
    ) -> abstract_std::objects::ans_host::AnsHostResult<Self::Output> {
        match self.1.clone() {
            DexAnsAction::ProvideLiquidity { assets, max_spread } => {
                let mut asset_names = assets
                    .iter()
                    .cloned()
                    .map(|a| a.name)
                    .take(2)
                    .collect::<Vec<_>>();
                let assets = assets.resolve(querier, ans_host)?;

                let pool_address = pool_address(
                    self.0.clone(),
                    (asset_names.swap_remove(0), asset_names.swap_remove(0)),
                    querier,
                    ans_host,
                )?;
                Ok(DexRawAction::ProvideLiquidity {
                    pool: pool_address.into(),
                    assets: assets.into_iter().map(Into::into).collect(),
                    max_spread,
                })
            }
            DexAnsAction::WithdrawLiquidity { lp_token } => {
                let lp_asset = lp_token.resolve(querier, ans_host)?;

                let lp_pairing: DexAssetPairing = AnsEntryConvertor::new(
                    AnsEntryConvertor::new(lp_token.name.clone()).lp_token()?,
                )
                .dex_asset_pairing()?;

                let mut pool_ids = lp_pairing.resolve(querier, ans_host)?;
                // TODO: when resolving if there are more than one, get the metadata and choose the one matching the assets
                if pool_ids.len() != 1 {
                    return Err(AnsHostError::QueryFailed {
                        method_name: "lp_pairing.resolve".to_string(),
                        error: StdError::generic_err(format!(
                            "There are {} pairings for the given LP token",
                            pool_ids.len()
                        )),
                    });
                }

                let pool_address = pool_ids.pop().unwrap().pool_address;
                Ok(DexRawAction::WithdrawLiquidity {
                    pool: pool_address.into(),
                    lp_token: lp_asset.into(),
                })
            }
            DexAnsAction::Swap {
                offer_asset,
                mut ask_asset,
                max_spread,
                belief_price,
            } => {
                let AnsAsset {
                    name: mut offer_asset,
                    amount: offer_amount,
                } = offer_asset.clone();
                offer_asset.format();
                ask_asset.format();

                let offer_asset_info = offer_asset.resolve(querier, ans_host)?;
                let ask_asset_info = ask_asset.resolve(querier, ans_host)?;

                let pool_address = pool_address(
                    self.0.clone(),
                    (offer_asset.clone(), ask_asset.clone()),
                    querier,
                    ans_host,
                )?;
                let offer_asset = Asset::new(offer_asset_info, offer_amount);

                Ok(DexRawAction::Swap {
                    pool: pool_address.into(),
                    offer_asset: offer_asset.into(),
                    ask_asset: ask_asset_info.into(),
                    max_spread,
                    belief_price,
                })
            }
        }
    }
}
