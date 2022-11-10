use abstract_api::ApiContract;
use abstract_os::api::{ExecuteMsg, InstantiateMsg, QueryMsg};
use cosmwasm_std::{entry_point, Binary, Deps, DepsMut, Env, MessageInfo, Response};
use cosmwasm_std::{Empty, StdResult};

use abstract_os::tendermint_staking::RequestMsg;
use abstract_sdk::{tendermint_staking::*, ExecuteEndpoint};
use abstract_sdk::{InstantiateEndpoint, OsExecute, QueryEndpoint};

use crate::error::TendermintStakeError;

use abstract_os::TENDERMINT_STAKING;
const CONTRACT_VERSION: &str = env!("CARGO_PKG_VERSION");

pub type TendermintStakeApi<'a> = ApiContract<TendermintStakeError, RequestMsg>;
pub type TendermintStakeResult = Result<Response, TendermintStakeError>;

const STAKING_API: TendermintStakeApi<'static> =
    TendermintStakeApi::new(TENDERMINT_STAKING, CONTRACT_VERSION).with_execute(handle_request);

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn instantiate(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
    msg: InstantiateMsg,
) -> TendermintStakeResult {
    STAKING_API.instantiate(deps, env, info, msg)
}

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn execute(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
    msg: ExecuteMsg<RequestMsg>,
) -> TendermintStakeResult {
    STAKING_API.execute(deps, env, info, msg)
}

pub fn handle_request(
    deps: DepsMut,
    _env: Env,
    _info: MessageInfo,
    api: TendermintStakeApi,
    msg: RequestMsg,
) -> TendermintStakeResult {
    let msg = match msg {
        RequestMsg::Delegate { validator, amount } => api.os_execute(
            deps.as_ref(),
            vec![delegate_to(&deps.querier, &validator, amount.u128())?],
        ),
        RequestMsg::UndelegateFrom { validator, amount } => {
            let undelegate_msg = match amount {
                Some(amount) => undelegate_from(&deps.querier, &validator, amount.u128())?,
                None => undelegate_all_from(&deps.querier, api.target()?, &validator)?,
            };
            api.os_execute(deps.as_ref(), vec![undelegate_msg])
        }
        RequestMsg::UndelegateAll {} => {
            api.os_execute(deps.as_ref(), undelegate_all(&deps.querier, api.target()?)?)
        }

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
                    api.target()?,
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
            withdraw_all_rewards(&deps.querier, api.target()?)?,
        ),
    }?;
    Ok(Response::new().add_submessage(msg))
}

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn query(deps: Deps, env: Env, msg: QueryMsg<Empty>) -> StdResult<Binary> {
    STAKING_API.query(deps, env, msg)
}
