use std::{env::set_var, sync::Arc};

use abstract_account::absacc::auth::AddAuthenticator;
use abstract_client::AbstractClient;
use cw_orch::{
    daemon::{
        networks::{xion::XION_NETWORK, XION_TESTNET_1},
        senders::CosmosSender,
        CosmosOptions, Daemon, TxSender, RUNTIME,
    },
    prelude::*,
};
use networks::ChainKind;

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
    set_var("RUST_LOG", "info");
    env_logger::init();

    let xiond = Daemon::builder(LOCAL_XION)
        .build_sender(CosmosOptions::default().mnemonic(LOCAL_MNEMONIC))?;

    let wallet = xiond.sender();

    let abstr = AbstractClient::builder(xiond).build()?;

    // normal account
    let account = abstr.account_builder().build()?;

    let code_id = account.as_ref().code_id()?;

    let Secp256k1 = secp256k1::Secp256k1::new();

    let create_msg = proto::MsgRegisterAccount {
        sender: wallet.pub_addr_str(),
        code_id,
        msg: to_json_binary(&abstract_std::account::InstantiateMsg {
            authenticator: AddAuthenticator::Ed25519 { id: 1, pubkey: wallet.private_key.public_key(secp), signature: () },
            name: "test".to_string(),
            account_id: None,
            // TODO: add new type for external Authenticator 
            owner: abstract_client::GovernanceDetails::Renounced {  },
            namespace: None,
            install_modules: None,
            description: None,
            link: None,
            module_factory_address: None,
            version_control_address: None,
        }),
        funds: vec![],
        salt: vec![],
    };

    
    RUNTIME.block_on(wallet.commit_tx_any(msgs, memo)


    Ok(())
}


mod proto {

    use cosmos_sdk_proto::cosmos;
    use cosmwasm_std::{AnyMsg, CosmosMsg};
    use prost::{Message, Name};

    #[derive(Clone, PartialEq, prost::Message)]
    pub struct MsgRegisterAccount {
        #[prost(string, tag = "1")]
        pub sender: String,

        #[prost(uint64, tag = "2")]
        pub code_id: u64,

        #[prost(bytes = "vec", tag = "3")]
        pub msg: Vec<u8>,

        #[prost(message, repeated, tag = "4")]
        pub funds: Vec<cosmos::base::v1beta1::Coin>,

        #[prost(bytes = "vec", tag = "5")]
        pub salt: Vec<u8>,
    }

    impl From<MsgRegisterAccount> for CosmosMsg {
        fn from(msg: MsgRegisterAccount) -> Self {
            let any_msg: AnyMsg = AnyMsg {
                type_url: MsgRegisterAccount::type_url(),
                value: msg.encode_to_vec().into(),
            };
            CosmosMsg::Any(any_msg)
        }
    }

    impl Name for MsgRegisterAccount {
        const NAME: &'static str = "MsgRegisterAccount";
        const PACKAGE: &'static str = "abstractaccount.v1";
    }

    #[derive(Clone, PartialEq, prost::Message)]
    pub struct MsgRegisterAccountResponse {
        #[prost(string, tag = "1")]
        pub address: String,

        #[prost(bytes = "vec", tag = "2")]
        pub data: Vec<u8>,
    }

    impl Name for MsgRegisterAccountResponse {
        const NAME: &'static str = "MsgRegisterAccountResponse";
        const PACKAGE: &'static str = "abstractaccount.v1";
    }
}