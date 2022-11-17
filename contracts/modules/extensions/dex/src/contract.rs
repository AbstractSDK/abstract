use abstract_extension::ExtensionContract;
use abstract_sdk::os::{
    dex::{DexAction, DexName, DexQueryMsg, DexRequestMsg, IBC_DEX_ID},
    extension::{ExecuteMsg, InstantiateMsg, QueryMsg},
    ibc_client::CallbackInfo,
    objects::AnsAsset,
    EXCHANGE,
};
use abstract_sdk::{
    base::endpoints::{ExecuteEndpoint, InstantiateEndpoint, QueryEndpoint},
    feature_objects::AnsHost,
    AnsInterface, IbcInterface, Resolve,
};
#[cfg(not(feature = "library"))]
use cosmwasm_std::entry_point;
use cosmwasm_std::{
    to_binary, Binary, Coin, Deps, DepsMut, Empty, Env, MessageInfo, Response, StdError, StdResult,
};

use crate::{
    commands::LocalDex, dex_trait::Identify, error::DexError, queries::simulate_swap, DEX,
};
const CONTRACT_VERSION: &str = env!("CARGO_PKG_VERSION");

pub type DexExtension = ExtensionContract<DexError, DexRequestMsg, Empty, DexQueryMsg>;
pub type DexResult = Result<Response, DexError>;

pub const DEX_EXTENSION: DexExtension = DexExtension::new(EXCHANGE, CONTRACT_VERSION)
    .with_execute(handle_request)
    .with_query(query_handler);

const ACTION_RETRIES: u8 = 3;

// Supported exchanges on Juno
#[cfg(feature = "juno")]
pub use crate::exchanges::junoswap::{JunoSwap, JUNOSWAP};

#[cfg(any(feature = "juno", feature = "terra"))]
pub use crate::exchanges::loop_dex::{Loop, LOOP};

#[cfg(feature = "terra")]
pub use crate::exchanges::terraswap::{Terraswap, TERRASWAP};

#[cfg(any(feature = "juno", feature = "osmosis"))]
pub use crate::exchanges::osmosis::{Osmosis, OSMOSIS};

pub(crate) fn identify_exchange(value: &str) -> Result<&'static dyn Identify, DexError> {
    match value {
        #[cfg(feature = "juno")]
        JUNOSWAP => Ok(&JunoSwap {}),
        #[cfg(feature = "juno")]
        OSMOSIS => Ok(&Osmosis {
            local_proxy_addr: None,
        }),
        #[cfg(any(feature = "juno", feature = "terra"))]
        LOOP => Ok(&Loop {}),
        #[cfg(feature = "terra")]
        TERRASWAP => Ok(&Terraswap {}),
        _ => Err(DexError::UnknownDex(value.to_owned())),
    }
}

pub(crate) fn resolve_exchange(value: &str) -> Result<&'static dyn DEX, DexError> {
    match value {
        #[cfg(feature = "juno")]
        JUNOSWAP => Ok(&JunoSwap {}),
        // #[cfg(feature = "osmosis")]
        // OSMOSIS => Ok(&Osmosis {
        //     local_proxy_addr: None,
        // }),
        #[cfg(any(feature = "juno", feature = "terra"))]
        LOOP => Ok(&Loop {}),
        #[cfg(feature = "terra")]
        TERRASWAP => Ok(&Terraswap {}),
        _ => Err(DexError::ForeignDex(value.to_owned())),
    }
}

// Supported exchanges on XXX
// ...
#[cfg_attr(not(feature = "library"), entry_point)]
pub fn instantiate(deps: DepsMut, env: Env, info: MessageInfo, msg: InstantiateMsg) -> DexResult {
    DEX_EXTENSION.instantiate(deps, env, info, msg)
}

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn execute(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
    msg: ExecuteMsg<DexRequestMsg>,
) -> DexResult {
    DEX_EXTENSION.execute(deps, env, info, msg)
}

pub fn handle_request(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
    extension: DexExtension,
    msg: DexRequestMsg,
) -> DexResult {
    let DexRequestMsg {
        dex: dex_name,
        action,
    } = msg;
    let exchange = identify_exchange(&dex_name)?;
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
    let exchange = resolve_exchange(&exchange)?;
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
    let ans = extension.ans(deps.as_ref());
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

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn query(deps: Deps, env: Env, msg: QueryMsg<DexQueryMsg>) -> StdResult<Binary> {
    DEX_EXTENSION.query(deps, env, msg)
}

fn query_handler(deps: Deps, env: Env, _app: &DexExtension, msg: DexQueryMsg) -> StdResult<Binary> {
    match msg {
        DexQueryMsg::SimulateSwap {
            offer_asset,
            ask_asset,
            dex,
        } => simulate_swap(deps, env, offer_asset, ask_asset, dex.unwrap()).map_err(Into::into),
    }
}

fn resolve_assets_to_transfer(
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
            info: lp_token.to_owned(),
            amount: amount.to_owned(),
        })?]),
        DexAction::Swap { offer_asset, .. } => Ok(vec![offer_to_coin(offer_asset)?]),
        DexAction::CustomSwap { offer_assets, .. } => {
            let coins: Result<Vec<Coin>, _> = offer_assets.iter().map(offer_to_coin).collect();
            coins
        }
    }
}
