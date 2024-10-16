use abstract_interface::AccountI;
use cosmos_sdk_proto::Any;
use cw_orch::daemon::networks::XION_TESTNET_1;
use cw_orch::prelude::*;
use cw_orch_interchain::prelude::*;
use flate2::{write, Compression};
use prost::Name;
use std::io::Write;
use xion_sdk_proto::cosmos::gov::v1::MsgSubmitProposalResponse;
use xion_sdk_proto::cosmwasm::wasm::v1::{AccessConfig, AccessType};
use xion_sdk_proto::cosmwasm::wasm::v1::{MsgStoreCode, StoreCodeProposal};
use xionrs::tx::MessageExt;

fn main() -> cw_orch::anyhow::Result<()> {
    dotenv::dotenv()?;
    env_logger::init();
    let xion = Daemon::builder(XION_TESTNET_1).build()?;
    println!("Sender : {} ", xion.sender_addr());
    let wasm_path = AccountI::<Daemon>::wasm(&XION_TESTNET_1.into());
    let file_contents = std::fs::read(wasm_path.path())?;
    let mut e = write::GzEncoder::new(Vec::new(), Compression::default());
    e.write_all(&file_contents)?;
    let wasm_byte_code = e.finish()?;

    let instantiate_permission = Some(AccessConfig {
        permission: AccessType::Everybody as i32,
        addresses: vec![],
        address: "".to_string(),
    });
    let msg_store_code = MsgStoreCode {
        sender: "xion10d07y265gmmuvt4z0w9aw880jnsr700jctf8qc".to_string(),
        wasm_byte_code: wasm_byte_code.clone(),
        instantiate_permission: instantiate_permission.clone(),
    };

    let msg_proposal = MsgSubmitProposal {
        messages: vec![
            xion_sdk_proto::Any{
                value: msg_store_code.to_bytes()?,
                type_url: MsgStoreCode::type_url()

            }
        ],
        initial_deposit: vec![xion_sdk_proto::cosmos::base::v1beta1::Coin{
            amount: 10_000_000u128.to_string(),
            denom:  "uxion".to_string()
        }],
        proposer: xion.sender_addr().to_string(),
        metadata: "https://github.com/AbstractSDK/abstract/tree/update/xion-abstractv0.24.1-beta.1-proposal".to_string(),
        title: "Upload Abstract Account v0.24.1-beta.1".to_string(),
        summary: "Upload Abstract Account v0.24.1-beta.1".to_string(),
    };

    xion.commit_any::<MsgSubmitProposalResponse>(
        vec![prost_types::Any {
            type_url: xion_sdk_proto::cosmos::gov::v1::MsgSubmitProposal::type_url(),
            value: msg_proposal.to_bytes()?,
        }],
        None,
    )?;

    Ok(())
}

/// This is copied from (https://github.com/burnt-labs/cosmos-rust/blob/d3b51db49b894f1c6b7836bb0a7b14f54f1dfb26/cosmos-sdk-proto/src/prost/cosmos-sdk/cosmos.gov.v1.rs#L400)
/// Because the proto is not up-to-date with the chain proto (something like https://github.com/cosmos/cosmos-sdk/blob/abaccb4d4b1f0ec0dd1f4b3df2aeb05bf3fb3e5d/x/gov/proto/cosmos/gov/v1/tx.proto#L66)
/// MsgSubmitProposal defines an sdk.Msg type that supports submitting arbitrary
/// proposal Content.
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct MsgSubmitProposal {
    #[prost(message, repeated, tag = "1")]
    pub messages: ::prost::alloc::vec::Vec<xion_sdk_proto::Any>,
    #[prost(message, repeated, tag = "2")]
    pub initial_deposit: ::prost::alloc::vec::Vec<xion_sdk_proto::cosmos::base::v1beta1::Coin>,
    #[prost(string, tag = "3")]
    pub proposer: ::prost::alloc::string::String,
    /// metadata is any arbitrary metadata attached to the proposal.
    #[prost(string, tag = "4")]
    pub metadata: ::prost::alloc::string::String,
    #[prost(string, tag = "5")]
    pub title: ::prost::alloc::string::String,
    #[prost(string, tag = "6")]
    pub summary: ::prost::alloc::string::String,
}
