use std::{num::NonZeroU32, str::FromStr};

use abstract_sdk::{
    feature_objects::{AnsHost, RegistryContract},
    Resolve,
};
use abstract_std::{
    ibc::PACKET_LIFETIME,
    ica_client::EVM_NOTE_ID,
    native_addrs,
    objects::{module::ModuleInfo, ChannelEntry, ContractEntry, TruncatedChainId},
};
use alloy::primitives::U256;
use alloy_sol_types::SolValue;
use cosmwasm_std::{
    wasm_execute, Addr, Binary, Coin, CosmosMsg, Deps, Env, HexBinary, QuerierWrapper, StdError,
    Uint256, Uint64, WasmMsg,
};
use evm_note::msg::{CallbackRequest, EvmMsg};

use crate::{contract::IcaClientResult, error::IcaClientError};

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
    account_address: &Addr,
    mut funds: Vec<Coin>,
    receiver: Option<Binary>,
    memo: Option<String>,
    salt: Uint256,
) -> IcaClientResult<CosmosMsg> {
    if funds.len() != 1 {
        panic!("Currently only single funds supported");
    }
    let funds = funds.pop().unwrap();
    // Identify the remote recipient for the funds
    let abstract_code_id =
        native_addrs::abstract_code_id(&deps.querier, env.contract.address.clone())?;

    let receiver: Vec<u8> = match receiver {
        Some(r) => r.into(),
        None => {
            let registry = RegistryContract::new(deps, abstract_code_id)?;
            let note_addr = evm_note_addr(&registry, &deps.querier)?;

            // If state objects will be public on evm_note
            let remote_acc: Option<String> = deps.querier.query_wasm_smart(
                &note_addr,
                &evm_note::msg::QueryMsg::RemoteAddress {
                    local_address: env.contract.address.to_string(),
                },
            )?;

            match remote_acc {
                Some(remote) => remote.into_bytes(),
                None => return Err(IcaClientError::NoRecipient {}),
            }
        }
    };

    let ans_host = AnsHost::new(deps, abstract_code_id)?;

    // Resolve the transfer channel id for the given chain
    let ucs_channel_entry = ChannelEntry {
        connected_chain: evm_chain.clone(),
        protocol: types::UCS03_ZKGM_PROTOCOL.to_string(),
    };
    let channel_id = ucs_channel_entry.resolve(&deps.querier, &ans_host)?;
    let channel_id =
        NonZeroU32::from_str(&channel_id).map_err(|e| StdError::generic_err(e.to_string()))?;

    // Resolve the transfer channel id for the given chain
    let ucs_contract_entry = ContractEntry {
        contract: types::UCS03_ZKGM_CONTRACT.to_string(),
        protocol: types::UCS03_ZKGM_PROTOCOL.to_string(),
    };
    let ucs03_zkgm_addr = ucs_contract_entry.resolve(&deps.querier, &ans_host)?;

    let quote_token: ucs03_zkgm::msg::PredictWrappedTokenResponse = deps.querier.query_wasm_smart(
        &ucs03_zkgm_addr,
        &ucs03_zkgm::msg::QueryMsg::PredictWrappedToken {
            path: U256::ZERO.to_string(),
            channel_id: channel_id.into(),
            token: funds.denom.clone().into_bytes().into(),
        },
    )?;

    let minter: Addr = deps
        .querier
        .query_wasm_smart(&ucs03_zkgm_addr, &ucs03_zkgm::msg::QueryMsg::GetMinter {})?;

    let metadata_response: ucs03_zkgm_token_minter_api::MetadataResponse =
        deps.querier.query_wasm_smart(
            &minter,
            &ucs03_zkgm_token_minter_api::QueryMsg::Metadata {
                denom: funds.denom.clone(),
            },
        )?;

    let timeout_timestamp = env.block.time.plus_days(2).nanos();
    let amount: U256 = funds.amount.u128().try_into().unwrap();

    // Construct instruction
    let instruction = ucs03_zkgm::com::Instruction {
        version: ucs03_zkgm::com::INSTR_VERSION_1,
        opcode: ucs03_zkgm::com::OP_FUNGIBLE_ASSET_ORDER,
        operand: ucs03_zkgm::com::FungibleAssetOrder {
            sender: account_address.to_string().into_bytes().into(),
            receiver: receiver.into(),
            base_token: funds.denom.clone().into_bytes().into(),
            base_amount: amount,
            base_token_symbol: metadata_response.symbol,
            base_token_name: metadata_response.name,
            base_token_decimals: metadata_response.decimals,
            // origin on this chain
            base_token_path: U256::ZERO,
            quote_token: quote_token.wrapped_token.into(),
            quote_amount: amount,
        }
        .abi_encode_params()
        .into(),
    };
    // Construct send packet on the ucs03zkgm
    let msg = ucs03_zkgm::msg::ExecuteMsg::Send {
        channel_id: channel_id.into(),
        // Setting non zero timeout height results in revert
        timeout_height: Uint64::zero(),
        timeout_timestamp: ibc_union_spec::Timestamp::from_nanos(timeout_timestamp),
        salt: salt.to_be_bytes().into(),
        instruction: instruction.abi_encode_params().into(),
    };

    let send_msg = wasm_execute(ucs03_zkgm_addr, &msg, vec![funds])?;

    Ok(send_msg.into())
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

    pub const UCS03_ZKGM_PROTOCOL: &str = "ucs03";
    pub const UCS03_ZKGM_CONTRACT: &str = "ucs03zkgm";

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
