use abstract_std::ibc::{IBCLifecycleComplete, ICS20PacketIdentifier};
use cosmwasm_std::{wasm_execute, DepsMut, Env, Response};

use crate::contract::AccountResult;
use crate::msg::ICS20_CALLBACKS;

pub fn ics20_hook_callback(deps: DepsMut, env: Env, msg: IBCLifecycleComplete) -> AccountResult {
    match msg {
        IBCLifecycleComplete::IBCAck {
            channel,
            sequence,
            ack: _,
            success,
        } => {
            let packet_identifier = ICS20PacketIdentifier {
                channel_id: channel,
                sequence,
            };

            // The acknowledgement has this structure with ibc hooks, we need to coed accordingly
            // https://github.com/cosmos/ibc-apps/blob/8cb681e31589bc90b47e0ab58173a579825fd56d/modules/ibc-hooks/wasm_hook.go#L119C1-L119C86
            let (outcome, stored_msgs) = if success {
                (
                    "result",
                    ICS20_CALLBACKS
                        .load(deps.storage, packet_identifier.clone())?
                        .into_iter()
                        .map(|msg| wasm_execute(&env.contract.address, &msg, vec![]))
                        .collect::<Result<Vec<_>, _>>()?,
                )
            } else {
                ("failure", vec![])
            };

            ICS20_CALLBACKS.remove(deps.storage, packet_identifier.clone());

            Ok(Response::new()
                .add_attribute("action", "ibc_source_callback")
                .add_attribute("outcome", outcome)
                .add_messages(stored_msgs))
        }
        IBCLifecycleComplete::IBCTimeout { channel, sequence } => {
            ICS20_CALLBACKS.remove(
                deps.storage,
                ICS20PacketIdentifier {
                    channel_id: channel,
                    sequence,
                },
            );
            Ok(Response::new()
                .add_attribute("action", "ibc_source_callback")
                .add_attribute("outcome", "timeout"))
        }
    }
}
