use cosmwasm_std::{entry_point, Binary, Deps, DepsMut, Env, MessageInfo, Response, StdResult};

use dao_os::treasury::dapp_base::commands::{self as dapp_base_commands, handle_base_init};
use dao_os::treasury::dapp_base::common::BaseDAppResult;
use dao_os::treasury::dapp_base::msg::BaseInstantiateMsg;
use dao_os::treasury::dapp_base::queries as dapp_base_queries;
use dao_os::treasury::dapp_base::state::{ADMIN, BASESTATE};

use crate::commands;
use crate::error::TerraswapError;
use crate::msg::{ExecuteMsg, QueryMsg};

pub type TerraswapResult = Result<Response, TerraswapError>;

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn instantiate(
    deps: DepsMut,
    _env: Env,
    info: MessageInfo,
    msg: BaseInstantiateMsg,
) -> BaseDAppResult {
    let base_state = handle_base_init(deps.as_ref(), msg)?;

    // Store the initial config
    BASESTATE.save(deps.storage, &base_state)?;

    // Setup the admin as the creator of the contract
    ADMIN.set(deps, Some(info.sender))?;

    Ok(Response::default())
}

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn execute(deps: DepsMut, env: Env, info: MessageInfo, msg: ExecuteMsg) -> TerraswapResult {
    match msg {
        ExecuteMsg::ProvideLiquidity {
            pool_id,
            main_asset_id,
            amount,
        } => commands::provide_liquidity(deps.as_ref(), info, main_asset_id, pool_id, amount),
        ExecuteMsg::DetailedProvideLiquidity {
            pool_id,
            assets,
            slippage_tolerance,
        } => commands::detailed_provide_liquidity(
            deps.as_ref(),
            info,
            assets,
            pool_id,
            slippage_tolerance,
        ),
        ExecuteMsg::WithdrawLiquidity {
            lp_token_id,
            amount,
        } => commands::withdraw_liquidity(deps.as_ref(), info, lp_token_id, amount),
        ExecuteMsg::SwapAsset {
            offer_id,
            pool_id,
            amount,
            max_spread,
            belief_price,
        } => commands::terraswap_swap(
            deps.as_ref(),
            env,
            info,
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
pub fn query(deps: Deps, _env: Env, msg: QueryMsg) -> StdResult<Binary> {
    match msg {
        QueryMsg::Base(message) => dapp_base_queries::handle_base_query(deps, message),
    }
}

/// Required to convert BaseDAppResult into TerraswapResult
/// Can't implement the From trait directly
fn from_base_dapp_result(result: BaseDAppResult) -> TerraswapResult {
    match result {
        Err(e) => Err(e.into()),
        Ok(r) => Ok(r),
    }
}
