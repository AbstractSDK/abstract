use abstract_adapter::sdk::features::AbstractNameService;
use abstract_adapter::std::objects::{AssetEntry, DexAssetPairing, PoolAddress};
use abstract_dex_standard::{
    ans_action::pool_address,
    msg::{
        DexExecuteMsg, DexFeesResponse, DexQueryMsg, GenerateMessagesResponse, SimulateSwapResponse,
    },
    DexError,
};
use cosmwasm_std::{to_json_binary, Binary, Deps, Env, StdError};

use crate::{
    contract::{DexAdapter, DexResult},
    exchanges::exchange_resolver::{self, resolve_exchange},
    handlers::query::exchange_resolver::is_over_ibc,
    state::DEX_FEES,
};
use cw_asset::{Asset, AssetInfo, AssetInfoBase};

pub fn query_handler(
    deps: Deps,
    env: Env,
    module: &DexAdapter,
    msg: DexQueryMsg,
) -> DexResult<Binary> {
    match msg {
        DexQueryMsg::SimulateSwapRaw {
            offer_asset,
            ask_asset,
            dex,
            pool,
        } => {
            let simulate_response = simulate_swap(
                deps,
                env,
                dex,
                pool.check(deps.api)?,
                offer_asset.check(deps.api, None)?,
                ask_asset.check(deps.api, None)?,
            )?;

            to_json_binary(&simulate_response).map_err(Into::into)
        }
        DexQueryMsg::GenerateMessages {
            message,
            addr_as_sender,
        } => {
            match message {
                DexExecuteMsg::Action { dex, action } => {
                    let (local_dex_name, is_over_ibc) = is_over_ibc(&env, &dex)?;
                    // if exchange is on an app-chain, execute the action on the app-chain
                    if is_over_ibc {
                        return Err(DexError::IbcMsgQuery);
                    }
                    let exchange = exchange_resolver::resolve_exchange(&local_dex_name)?;
                    let addr_as_sender = deps.api.addr_validate(&addr_as_sender)?;
                    let (messages, _) = crate::adapter::DexAdapter::resolve_dex_action(
                        module,
                        deps,
                        &env,
                        addr_as_sender,
                        action,
                        exchange,
                    )?;
                    to_json_binary(&GenerateMessagesResponse { messages }).map_err(Into::into)
                }
                _ => Err(DexError::InvalidGenerateMessage {}),
            }
        }
        DexQueryMsg::Fees {} => fees(deps),
        DexQueryMsg::SimulateSwap {
            offer_asset,
            ask_asset,
            dex,
        } => {
            let ans = module.name_service(deps);
            let cw_offer_asset = ans.query(&offer_asset)?;
            let cw_ask_asset = ans.query(&ask_asset)?;

            let pool_address = pool_address(
                &dex,
                (offer_asset.name.clone(), ask_asset.clone()),
                &deps.querier,
                ans.host(),
            )?;

            let simulate_response = simulate_swap(
                deps,
                env,
                dex.clone(),
                pool_address,
                cw_offer_asset,
                cw_ask_asset.clone(),
            )?;

            // We return ans assets here
            let resp = SimulateSwapResponse::<AssetEntry> {
                pool: DexAssetPairing::new(offer_asset.name.clone(), ask_asset.clone(), &dex),
                return_amount: simulate_response.return_amount,
                spread_amount: simulate_response.spread_amount,
                commission: if simulate_response.commission.0 == cw_ask_asset.into() {
                    (ask_asset, simulate_response.commission.1)
                } else {
                    (offer_asset.name, simulate_response.commission.1)
                },
                usage_fee: simulate_response.usage_fee,
            };
            to_json_binary(&resp).map_err(Into::into)
        }
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
    dex: String,
    pool: PoolAddress,
    mut offer_asset: Asset,
    ask_asset: AssetInfo,
) -> DexResult<SimulateSwapResponse<AssetInfoBase<String>>> {
    let exchange = resolve_exchange(&dex).map_err(|e| StdError::generic_err(e.to_string()))?;

    let pool_info = DexAssetPairing::new(
        offer_asset.info.clone().into(),
        ask_asset.clone().into(),
        exchange.name(),
    );

    // compute adapter fee
    let dex_fees = DEX_FEES.load(deps.storage)?;
    let adapter_fee = dex_fees.swap_fee().compute(offer_asset.amount);
    offer_asset.amount -= adapter_fee;

    let (return_amount, spread_amount, commission_amount, fee_on_input) = exchange
        .simulate_swap(deps, pool, offer_asset.clone(), ask_asset.clone())
        .map_err(|e| StdError::generic_err(e.to_string()))?;
    let commission_asset = if fee_on_input {
        ask_asset
    } else {
        offer_asset.info
    };

    let resp = SimulateSwapResponse {
        pool: pool_info,
        return_amount,
        spread_amount,
        commission: (commission_asset.into(), commission_amount),
        usage_fee: adapter_fee,
    };
    Ok(resp)
}
