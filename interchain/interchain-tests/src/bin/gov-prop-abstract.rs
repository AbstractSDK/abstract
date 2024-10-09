use abstract_client::{AbstractClient, Namespace};
use abstract_std::{
    ica_client::{IcaAction, IcaActionResult, QueryMsg},
    IBC_CLIENT, ICA_CLIENT,
};
use cosmwasm_std::coins;
use cw_orch::prelude::*;
use networks::union::UNION_TESTNET_8;
use prost_union::Message;
use prost_union::Name;
use protos::ibc::lightclients::wasm::v1::MsgStoreCode;

const TEST_ACCOUNT_NAMESPACE: &str = "testing";

fn main() -> cw_orch::anyhow::Result<()> {
    dotenv::dotenv()?;
    pretty_env_logger::init();
    // This is an integration test with Abstract And polytone EVM already deployed on Union

    // If it's not deployed, we can redeploy it here
    let chain_info = UNION_TESTNET_8;

    let chain = Daemon::builder(chain_info.clone()).build()?;

    let wasm = AccountI::wasm(&chain_info.into());

    chain.commit_any(
        vec![prost_types::Any {
            type_url: protos::cosmos::gov::v1beta1::MsgSubmitProposal::full_name(),
            value: protos::cosmos::gov::v1beta1::MsgSubmitProposal {
                content: Some(protos::google::protobuf::Any {
                    type_url: MsgStoreCode::type_url(),
                    value: MsgStoreCode {
                        signer: chain.sender_addr().to_string(),
                        wasm_byte_code: wasm,
                    }
                    .encode_to_vec(),
                }),
                initial_deposit: vec![],
                proposer: chain.sender_addr().to_string(),
            }
            .encode_to_vec(),
        }],
        None,
    )?;
    Ok(())
}
