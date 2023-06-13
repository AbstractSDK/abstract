use crate::handlers::query::exchange_resolver::is_over_ibc;

use crate::exchanges::exchange_resolver::resolve_exchange;

use crate::msg::{
    DexExecuteMsg, DexQueryMsg, GenerateMessagesResponse, OfferAsset, SimulateSwapResponse,
};
use crate::state::SWAP_FEE;
use crate::{
    contract::{DexAdapter, DexResult},
    exchanges::exchange_resolver,
};
use abstract_core::objects::{AssetEntry, DexAssetPairing};
use abstract_dex_adapter_traits::DexError;
use abstract_sdk::features::AbstractNameService;
use cosmwasm_std::{to_binary, Binary, Deps, Env, StdError};

pub fn query_handler(
    deps: Deps,
    env: Env,
    adapter: &DexAdapter,
    msg: DexQueryMsg,
) -> DexResult<Binary> {
    match msg {
        DexQueryMsg::SimulateSwap {
            offer_asset,
            ask_asset,
            dex,
        } => simulate_swap(deps, env, adapter, offer_asset, ask_asset, dex.unwrap()),
        DexQueryMsg::GenerateMessages { message } => {
            match message {
                DexExecuteMsg::Action { dex, action } => {
                    let (local_dex_name, is_over_ibc) = is_over_ibc(env, &dex)?;
                    // if exchange is on an app-chain, execute the action on the app-chain
                    if is_over_ibc {
                        return Err(DexError::IbcMsgQuery);
                    }
                    let exchange = exchange_resolver::resolve_exchange(&local_dex_name)?;
                    let (messages, _) = crate::adapter::DexAdapter::resolve_dex_action(
                        adapter, deps, action, exchange,
                    )?;
                    to_binary(&GenerateMessagesResponse { messages }).map_err(Into::into)
                }
                _ => Err(DexError::InvalidGenerateMessage {}),
            }
        }
    }
}

pub fn simulate_swap(
    deps: Deps,
    _env: Env,
    adapter: &DexAdapter,
    mut offer_asset: OfferAsset,
    mut ask_asset: AssetEntry,
    dex: String,
) -> DexResult<Binary> {
    let exchange = resolve_exchange(&dex).map_err(|e| StdError::generic_err(e.to_string()))?;
    let ans = adapter.name_service(deps);
    let fee = SWAP_FEE.load(deps.storage)?;

    // format input
    offer_asset.name.format();
    ask_asset.format();
    // get addresses
    let swap_offer_asset = ans.query(&offer_asset)?;
    let ask_asset_info = ans.query(&ask_asset)?;
    let pair_address = exchange
        .pair_address(
            deps,
            ans.host(),
            (offer_asset.name.clone(), ask_asset.clone()),
        )
        .map_err(|e| {
            StdError::generic_err(format!(
                "Failed to get pair address for {offer_asset:?} and {ask_asset:?}: {e}"
            ))
        })?;
    let pool_info =
        DexAssetPairing::new(offer_asset.name.clone(), ask_asset.clone(), exchange.name());

    // compute adapter fee
    let adapter_fee = fee.compute(offer_asset.amount);
    offer_asset.amount -= adapter_fee;

    let (return_amount, spread_amount, commission_amount, fee_on_input) = exchange
        .simulate_swap(deps, pair_address, swap_offer_asset, ask_asset_info)
        .map_err(|e| StdError::generic_err(e.to_string()))?;
    let commission_asset = if fee_on_input {
        ask_asset
    } else {
        offer_asset.name
    };
    let resp = SimulateSwapResponse {
        pool: pool_info,
        return_amount,
        spread_amount,
        commission: (commission_asset, commission_amount),
        usage_fee: adapter_fee,
    };
    to_binary(&resp).map_err(From::from)
}
