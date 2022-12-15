use crate::contract::{DexExtension, DexResult};
use crate::exchanges::exchange_resolver;
use crate::LocalDex;
use abstract_os::dex::{DexAction, DexExecuteMsg, DexName, IBC_DEX_ID};
use abstract_os::ibc_client::CallbackInfo;
use abstract_os::objects::ans_host::AnsHost;
use abstract_os::objects::AnsAsset;
use abstract_sdk::base::features::AbstractNameService;
use abstract_sdk::{IbcInterface, Resolve};
use cosmwasm_std::{
    to_binary, Coin, Deps, DepsMut, Env, MessageInfo, Response, StdError, StdResult,
};

const ACTION_RETRIES: u8 = 3;

pub fn execute_handler(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
    extension: DexExtension,
    msg: DexExecuteMsg,
) -> DexResult {
    let DexExecuteMsg {
        dex: dex_name,
        action,
    } = msg;
    let exchange = exchange_resolver::identify_exchange(&dex_name)?;
    // if exchange is on an app-chain, execute the action on the app-chain
    if exchange.over_ibc() {
        handle_ibc_extension_request(&deps, info, &extension, dex_name, &action)
    } else {
        // the action can be executed on the local chain
        handle_local_extension_request(deps, env, info, extension, action, dex_name)
    }
}

/// Handle an extension request that can be executed on the local chain
fn handle_local_extension_request(
    deps: DepsMut,
    _env: Env,
    _info: MessageInfo,
    extension: DexExtension,
    action: DexAction,
    exchange: String,
) -> DexResult {
    let exchange = exchange_resolver::resolve_exchange(&exchange)?;
    Ok(
        Response::new()
            .add_submessage(extension.resolve_dex_action(deps, action, exchange, false)?),
    )
}

fn handle_ibc_extension_request(
    deps: &DepsMut,
    info: MessageInfo,
    extension: &DexExtension,
    dex_name: DexName,
    action: &DexAction,
) -> DexResult {
    let host_chain = dex_name;
    let ans = extension.name_service(deps.as_ref());
    let ibc_client = extension.ibc_client(deps.as_ref());
    // get the to-be-sent assets from the action
    let coins = resolve_assets_to_transfer(deps.as_ref(), action, ans.host())?;
    // construct the ics20 call(s)
    let ics20_transfer_msg = ibc_client.ics20_transfer(host_chain.clone(), coins)?;
    // construct the action to be called on the host
    let action = abstract_sdk::os::ibc_host::HostAction::App {
        msg: to_binary(&action)?,
    };
    let maybe_contract_info = deps.querier.query_wasm_contract_info(info.sender.clone());
    let callback = if maybe_contract_info.is_err() {
        None
    } else {
        Some(CallbackInfo {
            id: IBC_DEX_ID.to_string(),
            receiver: info.sender.into_string(),
        })
    };
    let ibc_action_msg = ibc_client.host_action(host_chain, action, callback, ACTION_RETRIES)?;

    // call both messages on the proxy
    Ok(Response::new().add_messages(vec![ics20_transfer_msg, ibc_action_msg]))
}

pub(crate) fn resolve_assets_to_transfer(
    deps: Deps,
    dex_action: &DexAction,
    ans_host: &AnsHost,
) -> StdResult<Vec<Coin>> {
    // resolve asset to native asset
    let offer_to_coin = |offer: &AnsAsset| offer.resolve(&deps.querier, ans_host)?.try_into();

    match dex_action {
        DexAction::ProvideLiquidity { assets, .. } => {
            let coins: Result<Vec<Coin>, _> = assets.iter().map(offer_to_coin).collect();
            coins
        }
        DexAction::ProvideLiquiditySymmetric { .. } => Err(StdError::generic_err(
            "Cross-chain symmetric provide liquidity not supported.",
        )),
        DexAction::WithdrawLiquidity { lp_token, amount } => Ok(vec![offer_to_coin(&AnsAsset {
            name: lp_token.to_owned(),
            amount: amount.to_owned(),
        })?]),
        DexAction::Swap { offer_asset, .. } => Ok(vec![offer_to_coin(offer_asset)?]),
        DexAction::CustomSwap { offer_assets, .. } => {
            let coins: Result<Vec<Coin>, _> = offer_assets.iter().map(offer_to_coin).collect();
            coins
        }
    }
}
