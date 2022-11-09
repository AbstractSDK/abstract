use abstract_api::{ApiContract, ApiResult};
use abstract_os::{
    api::{BaseInstantiateMsg, ExecuteMsg, QueryMsg},
    dex::{DexAction, DexName, DexQueryMsg, DexRequestMsg, IBC_DEX_ID},
    ibc_client::CallbackInfo,
    objects::AssetEntry,
    EXCHANGE,
};
use abstract_sdk::{
    host_ibc_action, ics20_transfer, memory::Memory, AbstractExecute, MemoryOperation, Resolve,
};
#[cfg(not(feature = "library"))]
use cosmwasm_std::entry_point;
use cosmwasm_std::{
    to_binary, Binary, Coin, Deps, DepsMut, Env, MessageInfo, Response, StdResult, Uint128, StdError,
};
use cw_asset::Asset;

use crate::{
    commands::LocalDex, dex_trait::Identify, error::DexError, queries::simulate_swap, DEX,
};
const CONTRACT_VERSION: &str = env!("CARGO_PKG_VERSION");

pub type DexApi<'a> = ApiContract<'a, DexRequestMsg, DexError>;
pub type DexResult = Result<Response, DexError>;
pub const DEX_API: DexApi<'static> = DexApi::new();

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
        _ => Err(DexError::UnknownDex(value.to_owned())),
    }
}

// Supported exchanges on XXX
// ...
#[cfg_attr(not(feature = "library"), entry_point)]
pub fn instantiate(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
    msg: BaseInstantiateMsg,
) -> ApiResult {
    DexApi::instantiate(deps, env, info, msg, EXCHANGE, CONTRACT_VERSION, vec![])?;
    Ok(Response::default())
}

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn execute(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
    msg: ExecuteMsg<DexRequestMsg>,
) -> DexResult {
    DEX_API.execute(deps, env, info, msg, handle_api_request)
}

pub fn handle_api_request(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
    api: DexApi,
    msg: DexRequestMsg,
) -> DexResult {
    let DexRequestMsg {
        dex: dex_name,
        action,
    } = msg;
    let exchange = identify_exchange(&dex_name)?;
    // if exchange is on an app-chain, execute the action on the app-chain
    if exchange.over_ibc() {
        handle_ibc_api_request(&deps, info, &api, dex_name, &action)
    } else {
        // the action can be executed on the local chain
        handle_local_api_request(deps, env, info, api, action, dex_name)
    }
}

/// Handle an API request that can be executed on the local chain
fn handle_local_api_request(
    deps: DepsMut,
    _env: Env,
    _info: MessageInfo,
    api: DexApi,
    action: DexAction,
    exchange: String,
) -> DexResult {
    let exchange = resolve_exchange(&exchange)?;
    Ok(Response::new().add_submessage(api.resolve_dex_action(deps, action, exchange, false)?))
}

fn handle_ibc_api_request(
    deps: &DepsMut,
    info: MessageInfo,
    api: &DexApi,
    dex_name: DexName,
    action: &DexAction,
) -> DexResult {
    let host_chain = dex_name;
    let memory = api.load_memory(deps.storage)?;
    // get the to-be-sent assets from the action
    let coins = resolve_assets_to_transfer(deps.as_ref(), action, &memory)?;
    // construct the ics20 call(s)
    let ics20_transfer_msg = ics20_transfer(api.target()?, host_chain.clone(), coins)?;
    // construct the action to be called on the host
    let action = abstract_os::ibc_host::HostAction::App {
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
    let ibc_action_msg =
        host_ibc_action(api.target()?, host_chain, action, callback, ACTION_RETRIES)?;

    // call both messages on the proxy
    Ok(Response::new().add_messages(vec![ics20_transfer_msg, ibc_action_msg]))
}

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn query(deps: Deps, env: Env, msg: QueryMsg<DexQueryMsg>) -> Result<Binary, DexError> {
    DEX_API.handle_query(deps, env, msg, Some(query_handler))
}

fn query_handler(deps: Deps, env: Env, msg: DexQueryMsg) -> Result<Binary, DexError> {
    match msg {
        DexQueryMsg::SimulateSwap {
            offer_asset,
            ask_asset,
            dex,
        } => simulate_swap(deps, env, offer_asset, ask_asset, dex.unwrap()),
    }
}

fn resolve_assets_to_transfer(
    deps: Deps,
    dex_action: &DexAction,
    memory: &Memory,
) -> StdResult<Vec<Coin>> {
    // resolve asset to native asset
    let offer_to_coin = |offer: &(AssetEntry, Uint128)| {
        Asset {
            info: offer.0.resolve(deps, memory)?,
            amount: offer.1,
        }
        .try_into()
    };

    match dex_action {
        DexAction::ProvideLiquidity { assets, .. } => {
            let coins: Result<Vec<Coin>, _> = assets.iter().map(offer_to_coin).collect();
            coins
        }
        DexAction::ProvideLiquiditySymmetric { .. } => {
            Err(StdError::generic_err("Cross-chain symmetric provide liquidity not supported."))
        }
        DexAction::WithdrawLiquidity { lp_token, amount } => Ok(vec![offer_to_coin(&(
            lp_token.to_owned(),
            amount.to_owned(),
        ))?]),
        DexAction::Swap { offer_asset, .. } => Ok(vec![offer_to_coin(offer_asset)?]),
        DexAction::CustomSwap { offer_assets, .. } => {
            let coins: Result<Vec<Coin>, _> = offer_assets.iter().map(offer_to_coin).collect();
            coins
        }
    }
}
