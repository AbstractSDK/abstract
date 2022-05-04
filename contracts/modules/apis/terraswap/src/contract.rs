use cosmwasm_std::{
    entry_point, Binary, Deps, DepsMut, Empty, Env, MessageInfo, Response, StdResult,
};

use pandora_dapp_base::{DappContract, DappResult};
use pandora_os::modules::apis::terraswap::{ExecuteMsg, QueryMsg};
use pandora_os::pandora_dapp::msg::DappInstantiateMsg;

use crate::commands;
use crate::error::TerraswapError;

type TerraswapExtension = Option<Empty>;
pub type TerraswapDapp<'a> = DappContract<'a, TerraswapExtension, Empty>;
pub type TerraswapResult = Result<Response, TerraswapError>;

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn instantiate(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
    msg: DappInstantiateMsg,
) -> DappResult {
    TerraswapDapp::default().instantiate(deps, env, info, msg)?;

    Ok(Response::default())
}

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn execute(deps: DepsMut, env: Env, info: MessageInfo, msg: ExecuteMsg) -> TerraswapResult {
    let dapp = TerraswapDapp::default();
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
        } => commands::terraswap_swap(
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
        ExecuteMsg::Base(dapp_msg) => {
            from_base_dapp_result(dapp.execute(deps, env, info, dapp_msg))
        }
    }
}

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn query(deps: Deps, env: Env, msg: QueryMsg) -> StdResult<Binary> {
    match msg {
        QueryMsg::Base(dapp_msg) => TerraswapDapp::default().query(deps, env, dapp_msg),
    }
}

/// Required to convert BaseDAppResult into TerraswapResult
/// Can't implement the From trait directly
fn from_base_dapp_result(result: DappResult) -> TerraswapResult {
    match result {
        Err(e) => Err(e.into()),
        Ok(r) => Ok(r),
    }
}
