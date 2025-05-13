use abstract_sdk::{feature_objects::AnsHost, Resolve};
use abstract_std::{
    ibc::PACKET_LIFETIME,
    native_addrs,
    objects::{ChannelEntry, ContractEntry, TruncatedChainId},
};
use cosmwasm_std::{wasm_execute, Addr, Binary, Coin, CosmosMsg, Deps, Env, HexBinary, WasmMsg};
use evm_note::msg::{CallbackRequest, EvmMsg};

use crate::{contract::IcaClientResult, error::IcaClientError};

pub fn execute(
    note_addr: Addr,
    msgs: Vec<EvmMsg<String>>,
    callback: Option<CallbackRequest>,
) -> IcaClientResult<WasmMsg> {
    wasm_execute(
        note_addr,
        &evm_note::msg::ExecuteMsg::Execute {
            msgs,
            callback,
            timeout_seconds: PACKET_LIFETIME.into(),
        },
        vec![],
    )
    .map_err(Into::into)
}

pub fn send_funds(
    deps: Deps,
    env: &Env,
    note_addr: &Addr,
    evm_chain: &TruncatedChainId,
    funds: Vec<Coin>,
    receiver: Option<Binary>,
    memo: Option<String>,
) -> IcaClientResult<CosmosMsg> {
    // Identify the remote recipient for the funds
    let abstract_code_id =
        native_addrs::abstract_code_id(&deps.querier, env.contract.address.clone())?;

    let receiver: HexBinary = match receiver {
        Some(r) => r.into(),
        None => {
            // If state objects will be public on evm_note
            let remote_acc: Option<String> = deps.querier.query_wasm_smart(
                note_addr,
                &evm_note::msg::QueryMsg::RemoteAddress {
                    local_address: env.contract.address.to_string(),
                },
            )?;

            match remote_acc {
                Some(remote) => HexBinary::from_hex(&remote)?,
                None => return Err(IcaClientError::NoRecipient {}),
            }
        }
    };

    let ans_host = AnsHost::new(deps, abstract_code_id)?;

    // Resolve the transfer channel id for the given chain
    let ucs_channel_entry = ChannelEntry {
        connected_chain: evm_chain.clone(),
        protocol: types::UCS01_PROTOCOL.to_string(),
    };
    let ics20_channel_id = ucs_channel_entry.resolve(&deps.querier, &ans_host)?;

    // Resolve the transfer channel id for the given chain
    let ucs_contract_entry = ContractEntry {
        contract: types::UCS01_FORWARDER_CONTRACT.to_string(),
        protocol: types::UCS01_PROTOCOL.to_string(),
    };
    let ucs_forwarder_addr = ucs_contract_entry.resolve(&deps.querier, &ans_host)?;

    // Construct forward packet on the forwarder
    let forwarder_msg = wasm_execute(
        ucs_forwarder_addr.clone(),
        &types::Ucs01ForwarderExecuteMsg::Transfer {
            channel: ics20_channel_id.clone(),
            receiver: receiver.clone(),
            memo: memo.unwrap_or_default(),
            timeout: Some(3600),
        },
        funds,
    )?
    .into();

    Ok(forwarder_msg)
}

pub(crate) mod types {
    use super::*;

    pub const UCS01_PROTOCOL: &str = "ucs01";
    pub const UCS01_FORWARDER_CONTRACT: &str = "forwarder";

    #[cosmwasm_schema::cw_serde]
    pub enum Ucs01ForwarderExecuteMsg {
        Transfer {
            channel: String,
            receiver: HexBinary,
            memo: String,
            timeout: Option<u64>,
        },
    }
}
