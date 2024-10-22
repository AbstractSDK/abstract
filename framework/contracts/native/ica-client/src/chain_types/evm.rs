use abstract_sdk::{
    feature_objects::{AnsHost, RegistryContract},
    Resolve,
};
use abstract_std::ica_client::EVM_NOTE_ID;
use abstract_std::objects::{module::ModuleInfo, ChannelEntry, ContractEntry, TruncatedChainId};
use cosmwasm_std::{
    wasm_execute, Addr, Binary, Coin, CosmosMsg, Deps, Env, HexBinary, QuerierWrapper, WasmMsg,
};
use evm_note::msg::{CallbackRequest, EvmMsg};

use crate::{contract::IcaClientResult, error::IcaClientError, queries::PACKET_LIFETIME};

pub fn execute(
    querier: &QuerierWrapper,
    vc: &RegistryContract,
    msgs: Vec<EvmMsg<String>>,
    callback: Option<CallbackRequest>,
) -> IcaClientResult<WasmMsg> {
    let note_addr = evm_note_addr(vc, querier)?;

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
    evm_chain: &TruncatedChainId,
    funds: Vec<Coin>,
    receiver: Option<Binary>,
    memo: Option<String>,
) -> IcaClientResult<CosmosMsg> {
    // Identify the remote recipient for the funds
    let receiver: HexBinary = match receiver {
        Some(r) => r.into(),
        None => {
            let registry = RegistryContract::new(deps.api, env)?;
            let note_addr = evm_note_addr(&registry, &deps.querier)?;

            // If state objects will be public on evm_note
            let remote_acc: Option<String> = deps.querier.query_wasm_smart(
                &note_addr,
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

    let ans_host = AnsHost::new(deps.api, env)?;

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

fn evm_note_addr(vc: &RegistryContract, querier: &QuerierWrapper) -> IcaClientResult<Addr> {
    let evm_note_entry = ModuleInfo::from_id(
        EVM_NOTE_ID,
        abstract_std::ica_client::POLYTONE_EVM_VERSION.parse()?,
    )?;

    vc.query_module(evm_note_entry, querier)?
        .reference
        .unwrap_native()
        .map_err(Into::into)
}

pub mod types {
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
