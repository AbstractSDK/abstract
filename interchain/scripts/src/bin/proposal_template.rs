use cosmrs::proto::cosmos::gov::v1::MsgSubmitProposal;
use cosmrs::proto::cosmwasm::wasm::v1::{AccessConfig, AccessType, MsgUpdateParams, Params};
use cw_orch::prelude::*;
use networks::COSMOS_HUB_TESTNET;
use prost::Name;
use xionrs::tx::MessageExt;

fn main() -> cw_orch::anyhow::Result<()> {
    dotenv::dotenv()?;
    env_logger::init();
    let mut chain_info = COSMOS_HUB_TESTNET;
    chain_info.gas_price = 0.005;
    let chain = Daemon::builder(chain_info).build()?;

    let msg_update_params = MsgUpdateParams {
        authority: "cosmos10d07y265gmmuvt4z0w9aw880jnsr700j6zn9kn".to_string(),
        params: Some(Params {
            code_upload_access: Some(AccessConfig {
                permission: AccessType::AnyOfAddresses as i32,
                addresses: vec![
                    "cosmos1559zgk3mxm00qtr0zu2x5n4rh5vw704qaqj6ap".to_string(),
                    "cosmos14cl2dthqamgucg9sfvv4relp3aa83e4046yfy7".to_string(),
                ],
            }),
            instantiate_default_permission: AccessType::Everybody as i32,
        }),
    };

    let msg_proposal = MsgSubmitProposal {
        messages: vec![cosmrs::Any {
            value: msg_update_params.to_bytes()?,
            type_url: MsgUpdateParams::type_url(),
        }],
        initial_deposit: vec![cosmrs::proto::cosmos::base::v1beta1::Coin {
            amount: 50_000_000u128.to_string(),
            denom: "uatom".to_string(),
        }],
        proposer: chain.sender_addr().to_string(),
        metadata: "none".to_string(),
        title: "Give upload permissions to Abstract".to_string(),
        summary: "Give upload permissions to Abstract".to_string(),
        expedited: false,
    };

    let response = chain.commit_any(
        vec![prost_types::Any {
            type_url: MsgSubmitProposal::type_url(),
            value: msg_proposal.to_bytes()?,
        }],
        None,
    )?;
    println!("Tx Hash {:?}", response.txhash);

    Ok(())
}
