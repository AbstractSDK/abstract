use abstract_api::{ApiContract, ApiResult};
use abstract_os::api::{ApiInstantiateMsg, ApiInterfaceMsg};
use cosmwasm_std::{entry_point, Binary, Deps, DepsMut, Env, MessageInfo, Response, StdResult};

use abstract_os::tendermint_staking::{ExecuteMsg, QueryMsg};
use abstract_sdk::tendermint_staking::*;
use abstract_sdk::OsExecute;

use crate::error::TendermintStakeError;

pub type TendermintStakeApi<'a> = ApiContract<'a, ExecuteMsg>;
pub type TendermintStakeResult = Result<Response, TendermintStakeError>;

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn instantiate(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
    msg: ApiInstantiateMsg,
) -> ApiResult {
    TendermintStakeApi::default().instantiate(
        deps,
        env,
        info,
        msg,
        "tendermint_staking",
        "3.2.8",
    )?;

    Ok(Response::default())
}

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn execute(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
    msg: ApiInterfaceMsg<ExecuteMsg>,
) -> TendermintStakeResult {
    TendermintStakeApi::handle_request(deps, env, info, msg, handle_api_request)
}

pub fn handle_api_request(
    deps: DepsMut,
    _env: Env,
    _info: MessageInfo,
    api: ApiContract<ExecuteMsg>,
    msg: ExecuteMsg,
) -> TendermintStakeResult {
    match msg {
        ExecuteMsg::Delegate { validator, amount } => api.os_execute(
            deps.as_ref(),
            vec![delegate_to(&deps.querier, &validator, amount.u128())?],
        ),
        ExecuteMsg::UndelegateFrom { validator, amount } => {
            let undelegate_msg = match amount {
                Some(amount) => undelegate_from(&deps.querier, &validator, amount.u128())?,
                None => undelegate_all_from(&deps.querier, &api.request_destination, &validator)?,
            };
            api.os_execute(deps.as_ref(), vec![undelegate_msg])
        }
        ExecuteMsg::UndelegateAll {} => api.os_execute(
            deps.as_ref(),
            undelegate_all(&deps.querier, &api.request_destination)?,
        ),

        ExecuteMsg::Redelegate {
            source_validator,
            destination_validator,
            amount,
        } => {
            let redelegate_msg = match amount {
                Some(amount) => redelegate(
                    &deps.querier,
                    &source_validator,
                    &destination_validator,
                    amount.u128(),
                )?,
                None => redelegate_all(
                    &deps.querier,
                    &source_validator,
                    &destination_validator,
                    &api.request_destination,
                )?,
            };
            api.os_execute(deps.as_ref(), vec![redelegate_msg])
        }
        ExecuteMsg::SetWithdrawAddress {
            new_withdraw_address,
        } => api.os_execute(
            deps.as_ref(),
            vec![update_withdraw_address(deps.api, &new_withdraw_address)?],
        ),
        ExecuteMsg::WithdrawDelegatorReward { validator } => {
            api.os_execute(deps.as_ref(), vec![withdraw_rewards(&validator)])
        }
        ExecuteMsg::WithdrawAllRewards {} => api.os_execute(
            deps.as_ref(),
            withdraw_all_rewards(&deps.querier, &api.request_destination)?,
        ),
    }
    .map_err(From::from)
}

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn query(deps: Deps, env: Env, msg: QueryMsg) -> StdResult<Binary> {
    match msg {
        QueryMsg::Base(dapp_msg) => TendermintStakeApi::default().query(deps, env, dapp_msg),
    }
}
