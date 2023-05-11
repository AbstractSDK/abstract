use crate::msg::{TendermintStakingExecuteMsg, TendermintStakingQueryMsg};
use crate::staking::*;
use crate::{error::TendermintStakeError, TENDERMINT_STAKING};
use abstract_adapter::AdapterContract;
use abstract_sdk::Execution;
use cosmwasm_std::{DepsMut, Empty, Env, MessageInfo, Response};
const CONTRACT_VERSION: &str = env!("CARGO_PKG_VERSION");

pub type TendermintStakeAdapter = AdapterContract<
    TendermintStakeError,
    Empty,
    TendermintStakingExecuteMsg,
    TendermintStakingQueryMsg,
>;

const STAKING_ADAPTER: TendermintStakeAdapter =
    TendermintStakeAdapter::new(TENDERMINT_STAKING, CONTRACT_VERSION, None)
        .with_execute(handle_request);

pub type TendermintStakeResult = Result<Response, TendermintStakeError>;

// Export handlers
#[cfg(feature = "export")]
abstract_adapter::export_endpoints!(STAKING_ADAPTER, TendermintStakeAdapter);

pub fn handle_request(
    deps: DepsMut,
    _env: Env,
    _info: MessageInfo,
    adapter: TendermintStakeAdapter,
    msg: TendermintStakingExecuteMsg,
) -> TendermintStakeResult {
    let executor = adapter.executor(deps.as_ref());
    let msg = match msg {
        TendermintStakingExecuteMsg::Delegate { validator, amount } => {
            executor.execute(vec![delegate_to(&deps.querier, &validator, amount.u128())?])
        }
        TendermintStakingExecuteMsg::UndelegateFrom { validator, amount } => {
            let undelegate_msg = match amount {
                Some(amount) => undelegate_from(&deps.querier, &validator, amount.u128())?,
                None => undelegate_all_from(&deps.querier, adapter.target()?, &validator)?,
            };
            executor.execute(vec![undelegate_msg])
        }
        TendermintStakingExecuteMsg::UndelegateAll {} => {
            executor.execute(undelegate_all(&deps.querier, adapter.target()?)?)
        }

        TendermintStakingExecuteMsg::Redelegate {
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
                    adapter.target()?,
                )?,
            };
            executor.execute(vec![redelegate_msg])
        }
        TendermintStakingExecuteMsg::SetWithdrawAddress {
            new_withdraw_address,
        } => executor.execute(vec![update_withdraw_address(
            deps.api,
            &new_withdraw_address,
        )?]),
        TendermintStakingExecuteMsg::WithdrawDelegatorReward { validator } => {
            executor.execute(vec![withdraw_rewards(&validator)])
        }
        TendermintStakingExecuteMsg::WithdrawAllRewards {} => {
            executor.execute(withdraw_all_rewards(&deps.querier, adapter.target()?)?)
        }
    }?;
    Ok(Response::new().add_message(msg))
}
