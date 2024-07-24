use std::iter;

use abstract_adapter::sdk::Execution;
use abstract_adapter::AdapterContract;
use cosmwasm_std::{DepsMut, Empty, Env, MessageInfo, Response};

use crate::{
    error::TendermintStakeError,
    msg::{TendermintStakingExecuteMsg, TendermintStakingQueryMsg},
    staking::*,
    TENDERMINT_STAKING,
};
const CONTRACT_VERSION: &str = env!("CARGO_PKG_VERSION");

pub type TendermintStakeAdapter = AdapterContract<
    TendermintStakeError,
    Empty,
    TendermintStakingExecuteMsg,
    TendermintStakingQueryMsg,
>;

pub const STAKING_ADAPTER: TendermintStakeAdapter =
    TendermintStakeAdapter::new(TENDERMINT_STAKING, CONTRACT_VERSION, None)
        .with_execute(handle_request);

pub type TendermintStakeResult = Result<Response, TendermintStakeError>;

// Export handlers
#[cfg(feature = "export")]
abstract_adapter::export_endpoints!(STAKING_ADAPTER, TendermintStakeAdapter);

abstract_adapter::cw_orch_interface!(
    STAKING_ADAPTER,
    TendermintStakeAdapter,
    Empty,
    TMintStakingAdapter
);

pub fn handle_request(
    deps: DepsMut,
    _env: Env,
    _info: MessageInfo,
    adapter: TendermintStakeAdapter,
    msg: TendermintStakingExecuteMsg,
) -> TendermintStakeResult {
    let executor = adapter.executor(deps.as_ref());
    let msg = match msg {
        TendermintStakingExecuteMsg::Delegate { validator, amount } => executor.execute(
            iter::once(delegate_to(&deps.querier, &validator, amount.u128())?),
        ),
        TendermintStakingExecuteMsg::UndelegateFrom { validator, amount } => {
            let undelegate_msg = match amount {
                Some(amount) => undelegate_from(&deps.querier, &validator, amount.u128())?,
                None => undelegate_all_from(&deps.querier, adapter.target()?, &validator)?,
            };
            executor.execute(iter::once(undelegate_msg))
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
            executor.execute(iter::once(redelegate_msg))
        }
        TendermintStakingExecuteMsg::SetWithdrawAddress {
            new_withdraw_address,
        } => executor.execute(iter::once(update_withdraw_address(
            deps.api,
            &new_withdraw_address,
        )?)),
        TendermintStakingExecuteMsg::WithdrawDelegatorReward { validator } => {
            executor.execute(iter::once(withdraw_rewards(&validator)))
        }
        TendermintStakingExecuteMsg::WithdrawAllRewards {} => {
            executor.execute(withdraw_all_rewards(&deps.querier, adapter.target()?)?)
        }
    }?;
    Ok(Response::new().add_message(msg))
}
