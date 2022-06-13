use abstract_api::state::ApiInterfaceResponse;
use abstract_api::{ApiContract, ApiResult};
use cosmwasm_std::{entry_point, Binary, Deps, DepsMut, Env, MessageInfo, Response, StdResult};

use abstract_os::common_module::api_msg::{ApiInstantiateMsg, ApiRequestMsg, ApiInterfaceMsg};
use abstract_os::modules::apis::terraswap::{ExecuteMsg, QueryMsg};

use crate::commands;
use crate::error::TerraswapError;

pub type TerraswapApi<'a> = ApiContract<'a, ExecuteMsg>;
pub type TerraswapResult = Result<Response, TerraswapError>;

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn instantiate(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
    msg: ApiInstantiateMsg,
) -> ApiResult {
    TerraswapApi::default().instantiate(deps, env, info, msg, "terraswap", "3.2.8")?;

    Ok(Response::default())
}

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn execute(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
    msg: ApiInterfaceMsg<ExecuteMsg>,
) -> TerraswapResult {
    let mut api = TerraswapApi::default();
    let resp = api.handle_request(&mut deps, env,&info, msg)?;
    match resp {
        ApiInterfaceResponse::ExecResponse(resp) => Ok(resp),
        ApiInterfaceResponse::ProcessRequest(msg) => {
        match msg {
            ExecuteMsg::ProvideLiquidity {
                pool_id,
                main_asset_id,
                amount,
            } => commands::provide_liquidity(deps.as_ref(), info, api, main_asset_id, pool_id, amount),
            ExecuteMsg::DetailedProvideLiquidity {
                pool_id,
                assets,
                slippage_tolerance,
            } => commands::detailed_provide_liquidity(
                deps.as_ref(),
                info,
                api,
                assets,
                pool_id,
                slippage_tolerance,
            ),
            ExecuteMsg::WithdrawLiquidity {
                lp_token_id,
                amount,
            } => commands::withdraw_liquidity(deps.as_ref(), info, api, lp_token_id, amount),
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
                api,
                offer_id,
                pool_id,
                amount,
                max_spread,
                belief_price,
            ),
        }
    }
}

}

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn query(deps: Deps, env: Env, msg: QueryMsg) -> StdResult<Binary> {
    match msg {
        QueryMsg::Base(dapp_msg) => TerraswapApi::default().query(deps, env, dapp_msg),
    }
}

/// Required to convert BaseDAppResult into TerraswapResult
/// Can't implement the From trait directly
fn from_base_dapp_result(result: ApiResult) -> TerraswapResult {
    match result {
        Err(e) => Err(e.into()),
        Ok(r) => Ok(r),
    }
}
