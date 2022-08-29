use abstract_api::{ApiContract, ApiResult};
use abstract_os::api::{ApiInstantiateMsg, ApiInterfaceMsg, ApiQueryMsg};
use abstract_os::TENDERMINT_STAKING;
use cosmwasm_std::{entry_point, Binary, Deps, DepsMut, Env, MessageInfo, Response};

use abstract_os::tendermint_staking::RequestMsg;
use abstract_sdk::tendermint_staking::*;
use abstract_sdk::OsExecute;

use crate::error::TendermintStakeError;

pub type TendermintStakeApi<'a> = ApiContract<'a, RequestMsg>;
pub type TendermintStakeResult = Result<Response, TendermintStakeError>;
const CONTRACT_VERSION: &str = env!("CARGO_PKG_VERSION");

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn instantiate(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
    msg: ApiInstantiateMsg,
) -> ApiResult {
    TendermintStakeApi::instantiate(
        deps,
        env,
        info,
        msg,
        TENDERMINT_STAKING,
        CONTRACT_VERSION,
        vec![],
    )?;

    Ok(Response::default())
}

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn execute(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
    msg: ApiInterfaceMsg<RequestMsg>,
) -> TendermintStakeResult {
    TendermintStakeApi::handle_request(deps, env, info, msg, handle_api_request)
}

pub fn handle_api_request(
    deps: DepsMut,
    _env: Env,
    _info: MessageInfo,
    api: ApiContract<RequestMsg>,
    msg: RequestMsg,
) -> TendermintStakeResult {
    match msg {
        RequestMsg::Delegate { validator, amount } => api.os_execute(
            deps.as_ref(),
            vec![delegate_to(&deps.querier, &validator, amount.u128())?],
        ),
        RequestMsg::UndelegateFrom { validator, amount } => {
            let undelegate_msg = match amount {
                Some(amount) => undelegate_from(&deps.querier, &validator, amount.u128())?,
                None => undelegate_all_from(&deps.querier, &api.request_destination, &validator)?,
            };
            api.os_execute(deps.as_ref(), vec![undelegate_msg])
        }
        RequestMsg::UndelegateAll {} => api.os_execute(
            deps.as_ref(),
            undelegate_all(&deps.querier, &api.request_destination)?,
        ),

        RequestMsg::Redelegate {
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
        RequestMsg::SetWithdrawAddress {
            new_withdraw_address,
        } => api.os_execute(
            deps.as_ref(),
            vec![update_withdraw_address(deps.api, &new_withdraw_address)?],
        ),
        RequestMsg::WithdrawDelegatorReward { validator } => {
            api.os_execute(deps.as_ref(), vec![withdraw_rewards(&validator)])
        }
        RequestMsg::WithdrawAllRewards {} => api.os_execute(
            deps.as_ref(),
            withdraw_all_rewards(&deps.querier, &api.request_destination)?,
        ),
    }
    .map_err(From::from)
}

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn query(deps: Deps, env: Env, msg: ApiQueryMsg) -> Result<Binary, TendermintStakeError> {
    TendermintStakeApi::handle_query(deps, env, msg, None)
}
