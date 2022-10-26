use abstract_api::{ApiContract, ApiResult};
use abstract_os::{
    api::{BaseInstantiateMsg, ExecuteMsg, QueryMsg},
    dex::{ApiQueryMsg, DexAction, RequestMsg, IBC_DEX_ID},
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
    to_binary, Binary, Coin, Deps, DepsMut, Env, MessageInfo, Response, StdResult, Uint128,
};
use cw_asset::Asset;

use crate::{commands::*, error::DexError, queries::simulate_swap};
const CONTRACT_VERSION: &str = env!("CARGO_PKG_VERSION");

pub type DexApi<'a> = ApiContract<'a, RequestMsg, DexError>;
pub type DexResult = Result<Response, DexError>;
const DEX_API: DexApi<'static> = DexApi::new();
const ACTION_RETRIES: u8 = 3;

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
    msg: ExecuteMsg<RequestMsg>,
) -> DexResult {
    DEX_API.execute(deps, env, info, msg, handle_api_request)
}

pub fn handle_api_request(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
    api: DexApi,
    msg: RequestMsg,
) -> DexResult {
    let RequestMsg {
        dex: dex_name,
        action,
    } = msg;
    let exchange = resolve_exchange(&dex_name)?;
    // if exchange is on an app-chain,
    if exchange.over_ibc() {
        let host_chain = dex_name;
        let memory = api.load_memory(deps.storage)?;
        // get the to-be-sent assets from the action
        let coins = assets_to_transfer(deps.as_ref(), &action, &memory)?;
        // construct the ics20 call(s)
        let ics20_transfer_msg = ics20_transfer(api.target()?, host_chain.clone(), coins)?;
        // construct the action to be called on the host
        let action = abstract_os::ibc_host::HostAction::App {
            msg: to_binary(&action)?,
        };
        let ibc_action_msg = host_ibc_action(
            api.target()?,
            host_chain,
            action,
            Some(CallbackInfo {
                id: IBC_DEX_ID.to_string(),
                receiver: env.contract.address.to_string(),
            }),
            ACTION_RETRIES,
        )?;
        // call both messages on the proxy
        Ok(Response::new().add_messages(vec![ics20_transfer_msg, ibc_action_msg]))
    } else {
        // the action can be executed on the local chain
        match action {
            DexAction::ProvideLiquidity { assets, max_spread } => {
                if assets.len() < 2 {
                    return Err(DexError::TooFewAssets {});
                }
                provide_liquidity(deps.as_ref(), env, info, api, assets, exchange, max_spread)
            }
            DexAction::ProvideLiquiditySymmetric {
                offer_asset,
                paired_assets,
            } => {
                if paired_assets.is_empty() {
                    return Err(DexError::TooFewAssets {});
                }
                provide_liquidity_symmetric(
                    deps.as_ref(),
                    env,
                    info,
                    api,
                    offer_asset,
                    paired_assets,
                    exchange,
                )
            }
            DexAction::WithdrawLiquidity { lp_token, amount } => {
                withdraw_liquidity(deps.as_ref(), env, info, api, (lp_token, amount), exchange)
            }
            DexAction::Swap {
                offer_asset,
                ask_asset,
                max_spread,
                belief_price,
            } => swap(
                deps.as_ref(),
                env,
                info,
                api,
                offer_asset,
                ask_asset,
                exchange,
                max_spread,
                belief_price,
            ),
        }
    }
}

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn query(deps: Deps, env: Env, msg: QueryMsg<ApiQueryMsg>) -> Result<Binary, DexError> {
    DEX_API.handle_query(deps, env, msg, Some(query_handler))
}

fn query_handler(deps: Deps, env: Env, msg: ApiQueryMsg) -> Result<Binary, DexError> {
    match msg {
        ApiQueryMsg::SimulateSwap {
            offer_asset,
            ask_asset,
            dex,
        } => simulate_swap(deps, env, offer_asset, ask_asset, dex.unwrap()),
    }
}

fn assets_to_transfer(deps: Deps, dex_action: &DexAction, memory: &Memory) -> StdResult<Vec<Coin>> {
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
        DexAction::ProvideLiquiditySymmetric { offer_asset, .. } => {
            Ok(vec![offer_to_coin(offer_asset)?])
        }
        DexAction::WithdrawLiquidity { lp_token, amount } => Ok(vec![offer_to_coin(&(
            lp_token.to_owned(),
            amount.to_owned(),
        ))?]),
        DexAction::Swap { offer_asset, .. } => Ok(vec![offer_to_coin(offer_asset)?]),
    }
}
