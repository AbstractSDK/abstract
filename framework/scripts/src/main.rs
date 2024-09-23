use std::{env::set_var, sync::Arc};

use abstract_account::absacc::{auth::AddAuthenticator, proto::MsgRegisterAccount};
use abstract_client::AbstractClient;
use abstract_std::{
    objects::{
        module::{Module, ModuleInfo},
        module_reference::ModuleReference,
        salt::generate_instantiate_salt,
        AccountId,
    },
    version_control::Account,
    ACCOUNT,
};
use cosmwasm_std::{to_json_binary, Binary};
use cw_orch::{
    daemon::{
        networks::{xion::XION_NETWORK, XION_TESTNET_1},
        senders::CosmosSender,
        CosmosOptions, Daemon, TxSender, RUNTIME,
    },
    prelude::*,
};
use networks::ChainKind;
use xion_sdk_proto::{prost::Name, traits::Message};

// Xiond validator seed
const LOCAL_MNEMONIC: &str = "clinic tube choose fade collect fish original recipe pumpkin fantasy enrich sunny pattern regret blouse organ april carpet guitar skin work moon fatigue hurdle";

pub const LOCAL_XION: ChainInfo = ChainInfo {
    kind: ChainKind::Local,
    chain_id: "xion-devnet-1",
    gas_denom: "uxion",
    gas_price: 0.03,
    grpc_urls: &["http://localhost:9090"],
    network_info: XION_NETWORK,
    lcd_url: None,
    fcd_url: None,
};

fn main() -> anyhow::Result<()> {
    set_var("RUST_LOG", "debug");
    env_logger::init();

    let xiond = Daemon::builder(LOCAL_XION)
        .build_sender(CosmosOptions::default().mnemonic(LOCAL_MNEMONIC))?;

    let wallet = xiond.sender();

    let abstr = AbstractClient::new(xiond.clone())?;

    // normal account
    // let account = abstr.account_builder().build()?;

    // Signature for xion account
    let next_account = abstr.random_account_id()?;
    let account_module = ModuleInfo::from_id_latest(ACCOUNT)?;
    let account_addr = abstr
        .module_instantiate2_address_raw(&AccountId::local(next_account), account_module.clone())?;
    let salt = generate_instantiate_salt(&AccountId::local(next_account));
    // get the account number of the wallet
    let signing_key =
        cosmrs::crypto::secp256k1::SigningKey::from_slice(&wallet.private_key.raw_key()).unwrap();
    let signature = signing_key.sign(account_addr.as_bytes()).unwrap();
    let Module {
        reference: ModuleReference::Account(code_id),
        ..
    } = abstr.version_control().module(account_module.clone())?
    else {
        unreachable!()
    };

    let create_msg = MsgRegisterAccount {
        sender: wallet.pub_addr_str(),
        code_id,
        msg: to_json_binary(&abstract_std::account::InstantiateMsg {
            authenticator: Some(AddAuthenticator::Ed25519 {
                id: 1,
                pubkey: Binary::new(
                    wallet
                        .private_key
                        .public_key(&wallet.secp)
                        .raw_pub_key
                        .unwrap(),
                ),
                signature: Binary::new(signature.to_vec()),
            }),
            name: "test".to_string(),
            account_id: None,
            // TODO: add new type for external Authenticator
            owner: abstract_client::GovernanceDetails::Renounced {},
            namespace: None,
            install_modules: vec![],
            description: None,
            link: None,
            module_factory_address: abstr.module_factroy().addr_str()?,
            version_control_address: abstr.version_control().addr_str()?,
        })?
        .to_vec(),
        funds: vec![],
        salt: salt.to_vec(),
    };

    xiond.rt_handle.block_on(wallet.commit_tx_any(
        vec![cosmrs::Any {
            type_url: MsgRegisterAccount::type_url(),
            value: create_msg.encode_to_vec(),
        }],
        None,
    ))?;

    Ok(())
}

// mod proto {

//     use cosmos_sdk_proto::cosmos;
//     use cosmwasm_std::{AnyMsg, CosmosMsg};
//     use prost::{Message, Name};

//     #[derive(Clone, PartialEq, prost::Message)]
//     pub struct MsgRegisterAccount {
//         #[prost(string, tag = "1")]
//         pub sender: String,

//         #[prost(uint64, tag = "2")]
//         pub code_id: u64,

//         #[prost(bytes = "vec", tag = "3")]
//         pub msg: Vec<u8>,

//         #[prost(message, repeated, tag = "4")]
//         pub funds: Vec<cosmos::base::v1beta1::Coin>,

//         #[prost(bytes = "vec", tag = "5")]
//         pub salt: Vec<u8>,
//     }

//     impl From<MsgRegisterAccount> for CosmosMsg {
//         fn from(msg: MsgRegisterAccount) -> Self {
//             let any_msg: AnyMsg = AnyMsg {
//                 type_url: MsgRegisterAccount::type_url(),
//                 value: msg.encode_to_vec().into(),
//             };
//             CosmosMsg::Any(any_msg)
//         }
//     }

//     impl Name for MsgRegisterAccount {
//         const NAME: &'static str = "MsgRegisterAccount";
//         const PACKAGE: &'static str = "abstractaccount.v1";
//     }

//     #[derive(Clone, PartialEq, prost::Message)]
//     pub struct MsgRegisterAccountResponse {
//         #[prost(string, tag = "1")]
//         pub address: String,

//         #[prost(bytes = "vec", tag = "2")]
//         pub data: Vec<u8>,
//     }

//     impl Name for MsgRegisterAccountResponse {
//         const NAME: &'static str = "MsgRegisterAccountResponse";
//         const PACKAGE: &'static str = "abstractaccount.v1";
//     }
// }
