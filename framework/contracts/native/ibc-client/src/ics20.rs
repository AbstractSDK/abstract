use abstract_std::{
    ibc::{IBCLifecycleComplete, ICS20PacketIdentifier},
    ibc_client::state::ICS20_ACCOUNT_CALLBACKS,
};
use cosmwasm_std::{BankMsg, CosmosMsg, DepsMut, Env, Response, WasmMsg};

use crate::contract::IbcClientResult;

pub fn ics20_hook_callback(deps: DepsMut, _env: Env, msg: IBCLifecycleComplete) -> IbcClientResult {
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

            let (account_addr, coin, actions) =
                ICS20_ACCOUNT_CALLBACKS.load(deps.storage, packet_identifier.clone())?;

            // The acknowledgement has this structure with ibc hooks, we need to coed accordingly
            // https://github.com/cosmos/ibc-apps/blob/8cb681e31589bc90b47e0ab58173a579825fd56d/modules/ibc-hooks/wasm_hook.go#L119C1-L119C86
            let (outcome, stored_msgs) = if success {
                let actions = actions
                    .into_iter()
                    .map(|msg| {
                        WasmMsg::Execute {
                            contract_addr: account_addr.to_string(),
                            msg,
                            funds: vec![],
                        }
                        .into()
                    })
                    .collect::<Vec<_>>();
                ("result", actions)
            } else {
                // On failure return funds
                (
                    "failure",
                    vec![BankMsg::Send {
                        to_address: account_addr.to_string(),
                        amount: vec![coin],
                    }
                    .into()],
                )
            };

            ICS20_ACCOUNT_CALLBACKS.remove(deps.storage, packet_identifier.clone());

            Ok(Response::new()
                .add_attribute("action", "ibc_source_callback")
                .add_attribute("outcome", outcome)
                .add_messages::<CosmosMsg>(stored_msgs))
        }
        IBCLifecycleComplete::IBCTimeout { channel, sequence } => {
            let packet_identifier = ICS20PacketIdentifier {
                channel_id: channel,
                sequence,
            };
            let (account_addr, coin, _) =
                ICS20_ACCOUNT_CALLBACKS.load(deps.storage, packet_identifier.clone())?;

            ICS20_ACCOUNT_CALLBACKS.remove(deps.storage, packet_identifier);
            // On timeout return funds
            let msg = BankMsg::Send {
                to_address: account_addr.to_string(),
                amount: vec![coin],
            };
            Ok(Response::new()
                .add_attribute("action", "ibc_source_callback")
                .add_attribute("outcome", "timeout")
                .add_message(msg))
        }
    }
}
