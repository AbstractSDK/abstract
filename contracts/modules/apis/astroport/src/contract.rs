#![allow(unused_imports)]
#![allow(unused_variables)]

use cosmwasm_std::{
    entry_point, Binary, Deps, DepsMut, Empty, Env, MessageInfo, Response, StdResult,
};

use pandora_dapp_base::{DappContract, DappResult};
use pandora_os::core::proxy::proxy_assets::{get_identifier, ProxyAsset};
use pandora_os::modules::apis::astroport::{ExecuteMsg, QueryMsg};
use pandora_os::modules::dapp_base::commands::{self as dapp_base_commands, handle_base_init};
use pandora_os::modules::dapp_base::common::BaseDAppResult;
use pandora_os::modules::dapp_base::msg::BaseInstantiateMsg;
use pandora_os::modules::dapp_base::queries as dapp_base_queries;
use pandora_os::modules::dapp_base::state::{BaseState, ADMIN, BASESTATE};
use pandora_os::native::memory::item::Memory;
use pandora_os::pandora_dapp::msg::DappInstantiateMsg;

use crate::commands;
use crate::error::AstroportError;
use crate::msg::{ExecuteMsg, QueryMsg};

type AstroportExtension = Option<Empty>;
pub type AstroportDapp<'a> = DappContract<'a, AstroportExtension, Empty>;
pub type AstroportResult = Result<Response, AstroportError>;

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn instantiate(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
    msg: DappInstantiateMsg,
) -> BaseDAppResult {
    AstroportDapp::default().instantiate(deps, env, info, msg)?;

    Ok(Response::default())
}

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn execute(deps: DepsMut, env: Env, info: MessageInfo, msg: ExecuteMsg) -> AstroportResult {
    let dapp = AstroportDapp::default();
    match msg {
        ExecuteMsg::ProvideLiquidity {
            pool_id,
            main_asset_id,
            amount,
        } => commands::provide_liquidity(deps.as_ref(), info, dapp, main_asset_id, pool_id, amount),
        ExecuteMsg::DetailedProvideLiquidity {
            pool_id,
            assets,
            slippage_tolerance,
        } => commands::detailed_provide_liquidity(
            deps.as_ref(),
            info,
            dapp,
            assets,
            pool_id,
            slippage_tolerance,
        ),
        ExecuteMsg::WithdrawLiquidity {
            lp_token_id,
            amount,
        } => commands::withdraw_liquidity(deps.as_ref(), info, dapp, lp_token_id, amount),
        ExecuteMsg::SwapAsset {
            offer_id,
            pool_id,
            amount,
            max_spread,
            belief_price,
        } => commands::astroport_swap(
            deps.as_ref(),
            env,
            info,
            dapp,
            offer_id,
            pool_id,
            amount,
            max_spread,
            belief_price,
        ),
        ExecuteMsg::Base(message) => {
            from_base_dapp_result(dapp_base_commands::handle_base_message(deps, info, message))
        }
    }
}

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn query(deps: Deps, env: Env, msg: QueryMsg) -> StdResult<Binary> {
    match msg {
        QueryMsg::Base(dapp_msg) => AstroportDapp::default().query(deps, env, dapp_msg),
        // handle dapp-specific queries here
        // QueryMsg::Custom{} => queries::custom_query(),
    }
}

/// Required to convert BaseDAppResult into AstroportResult
/// Can't implement the From trait directly
fn from_base_dapp_result(result: DappResult) -> AstroportResult {
    match result {
        Err(e) => Err(e.into()),
        Ok(r) => Ok(r),
    }
}
