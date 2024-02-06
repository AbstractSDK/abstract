use abstract_core::objects::pool_id::UncheckedPoolAddress;
use abstract_dex_standard::{
    msg::{
        AskAsset, DexExecuteMsg, DexFeesResponse, DexQueryMsg, GenerateMessagesResponse,
        OfferAsset, SimulateSwapResponse,
    },
    DexError,
};
use cosmwasm_std::{to_json_binary, Binary, Deps, Env, StdError};

use crate::{
    adapter::DexAdapter as _,
    contract::{DexAdapter, DexResult},
    exchanges::exchange_resolver::{self, resolve_exchange},
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
            pool,
        } => simulate_swap(
            deps,
            env,
            adapter,
            dex.unwrap(),
            pool,
            offer_asset,
            ask_asset,
        ),
        DexQueryMsg::GenerateMessages {
            message,
            proxy_addr,
        } => {
            match message {
                DexExecuteMsg::Action { dex, action, pool } => {
                    let (local_dex_name, is_over_ibc) = is_over_ibc(env, &dex)?;
                    // if exchange is on an app-chain, execute the action on the app-chain
                    if is_over_ibc {
                        return Err(DexError::IbcMsgQuery);
                    }
                    let exchange = exchange_resolver::resolve_exchange(&local_dex_name)?;
                    let sender = deps.api.addr_validate(&proxy_addr)?;
                    let (messages, _) = crate::adapter::DexAdapter::resolve_dex_action(
                        adapter,
                        deps,
                        sender,
                        action,
                        exchange,
                        pool.map(|p| p.check(deps.api)).transpose()?,
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
    dex: String,
    pool: Option<UncheckedPoolAddress>,
    offer_asset: OfferAsset,
    ask_asset: AskAsset,
) -> DexResult<Binary> {
    let exchange = resolve_exchange(&dex).map_err(|e| StdError::generic_err(e.to_string()))?;

    let mut cw_offer_asset = adapter._get_offer_asset(deps, &offer_asset)?;
    let cw_ask_asset = adapter._get_ask_asset(deps, &ask_asset)?;

    let pool_address = adapter
        ._get_pool(
            deps,
            exchange.as_ref(),
            pool.map(|p| p.check(deps.api)).transpose()?,
            &offer_asset.clone().info(),
            &ask_asset,
        )
        .map_err(|e| {
            StdError::generic_err(format!(
                "Failed to get pair address for {offer_asset:?} and {ask_asset:?}: {e}"
            ))
        })?;
    let pool_info = (
        offer_asset.info(),
        ask_asset.clone(),
        exchange.name().to_string(),
    );

    // compute adapter fee
    let dex_fees = DEX_FEES.load(deps.storage)?;
    let adapter_fee = dex_fees.swap_fee().compute(offer_asset.amount());
    cw_offer_asset.amount = offer_asset.amount() - adapter_fee;

    let (return_amount, spread_amount, commission_amount, fee_on_input) = exchange
        .simulate_swap(deps, pool_address, cw_offer_asset, cw_ask_asset)
        .map_err(|e| StdError::generic_err(e.to_string()))?;
    let commission_asset = if fee_on_input {
        ask_asset
    } else {
        offer_asset.info()
    };

    let resp = SimulateSwapResponse {
        pool: pool_info,
        return_amount,
        spread_amount,
        commission: OfferAsset::from_asset(commission_asset, commission_amount),
        usage_fee: adapter_fee,
    };
    to_json_binary(&resp).map_err(From::from)
}
