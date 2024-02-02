use abstract_core::objects::{AssetEntry, DexAssetPairing};
use abstract_dex_standard::{
    msg::{
        DexExecuteMsg, DexFeesResponse, DexQueryMsg, GenerateMessagesResponse, OfferAsset,
        SimulateSwapResponse,
    },
    DexError,
};
use abstract_sdk::features::AbstractNameService;
use cosmwasm_std::{to_json_binary, Binary, Deps, Env, StdError};

use crate::{
    contract::{DexAdapter, DexResult},
    exchanges::{exchange_resolver, exchange_resolver::resolve_exchange},
    handlers::query::exchange_resolver::is_over_ibc,
    state::DEX_FEES,
};

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
        DexQueryMsg::GenerateMessages { message, sender } => {
            match message {
                DexExecuteMsg::Action { dex, action } => {
                    let (local_dex_name, is_over_ibc) = is_over_ibc(env, &dex)?;
                    // if exchange is on an app-chain, execute the action on the app-chain
                    if is_over_ibc {
                        return Err(DexError::IbcMsgQuery);
                    }
                    let exchange = exchange_resolver::resolve_exchange(&local_dex_name)?;
                    let sender = deps.api.addr_validate(&sender)?;
                    let (messages, _) = crate::adapter::DexAdapter::resolve_dex_action(
                        adapter, deps, sender, action, exchange,
                    )?;
                    to_json_binary(&GenerateMessagesResponse { messages }).map_err(Into::into)
                }
                _ => Err(DexError::InvalidGenerateMessage {}),
            }
        }
        DexQueryMsg::Fees {} => fees(deps),
    }
}

pub fn fees(deps: Deps) -> DexResult<Binary> {
    let dex_fees = DEX_FEES.load(deps.storage)?;
    let resp = DexFeesResponse {
        swap_fee: dex_fees.swap_fee(),
        recipient: dex_fees.recipient,
    };
    to_json_binary(&resp).map_err(Into::into)
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
    let dex_fees = DEX_FEES.load(deps.storage)?;

    // format input
    offer_asset.name.format();
    ask_asset.format();
    // get addresses
    let swap_offer_asset = ans.query(&offer_asset)?;
    let ask_asset_info = ans.query(&ask_asset)?;
    let pool_address = exchange
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
    let adapter_fee = dex_fees.swap_fee().compute(offer_asset.amount);
    offer_asset.amount -= adapter_fee;

    let (return_amount, spread_amount, commission_amount, fee_on_input) = exchange
        .simulate_swap(deps, pool_address, swap_offer_asset, ask_asset_info)
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
    to_json_binary(&resp).map_err(From::from)
}
