use std::{env::set_var, sync::Arc};

use abstract_client::{AbstractClient, Namespace};
use abstract_std::{
    account,
    objects::{module::ModuleInfo, salt::generate_instantiate_salt, AccountId},
    ACCOUNT,
};
use abstract_xion::AddAuthenticator;
use bitcoin::secp256k1::{All, Secp256k1, Signing};
use cosmwasm_std::to_json_vec;
use cosmwasm_std::{coins, to_json_binary, Binary};
use cw_orch::{
    daemon::{networks::xion::XION_NETWORK, Daemon, TxSender, RUNTIME},
    prelude::*,
};
use cw_orch_core::{environment::ChainInfoOwned, CwEnvError};
use cw_orch_daemon::senders::CosmosWalletKey;
use cw_orch_daemon::senders::{builder::SenderBuilder, CosmosSender};
use cw_orch_daemon::CosmosOptions;
use cw_orch_daemon::QuerySender;
use cw_orch_daemon::{
    env::{DaemonEnvVars, LOCAL_MNEMONIC_ENV_NAME, MAIN_MNEMONIC_ENV_NAME, TEST_MNEMONIC_ENV_NAME},
    keys::private::PrivateKey,
    queriers::Node,
    tx_broadcaster::assert_broadcast_code_cosm_response,
    tx_builder::TxBuilder,
    CosmTxResponse, DaemonError, GrpcChannel,
};
use networks::ChainKind;
use tonic::transport::Channel;
use xion_sdk_proto::abstract_account::v1::NilPubKey;
use xion_sdk_proto::cosmos::auth::v1beta1::QueryAccountRequest;
use xion_sdk_proto::traits::MessageExt;
use xion_sdk_proto::{cosmos::bank::v1beta1::MsgSend, prost::Name, traits::Message};
use xionrs::crypto::secp256k1::SigningKey;

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
    set_var("RUST_LOG", "info");
    dotenv::dotenv()?;
    env_logger::init();

    let xiond = Daemon::builder(LOCAL_XION)
        .mnemonic(LOCAL_MNEMONIC)
        .build()?;

    let wallet = xiond.sender();

    let abstract_sender = xiond.rt_handle.block_on(CosmosSender::new(
        &Arc::new(xiond.chain_info().to_owned()),
        CosmosOptions::default(),
    ))?;
    xiond.rt_handle.block_on(wallet.bank_send(
        &abstract_sender.address(),
        coins(1_000_000_000_000, "uxion"),
    ))?;

    let abstr = AbstractClient::new(xiond.clone())
        .or_else(|_| AbstractClient::builder(xiond.clone()).build());

    let abstr = abstr?;
    // Create the Abstract Account because it's needed for the fees for the dex module
    let maybe_account = abstr.fetch_account(Namespace::new("test")?);

    println!("Wallet Addr: {}", wallet.pub_addr_str());
    let account = match maybe_account {
        Ok(acc) => acc,
        Err(_) => {
            let next_account = abstr.random_account_id()?;
            let account_module = ModuleInfo::from_id_latest(ACCOUNT)?;
            let account_id = AccountId::local(next_account);
            let account_addr =
                abstr.module_instantiate2_address_raw(&account_id, account_module.clone())?;
            let salt = generate_instantiate_salt(&account_id);

            // get the account number of the wallet
            let signing_key =
                xionrs::crypto::secp256k1::SigningKey::from_slice(&wallet.private_key.raw_key())
                    .unwrap();
            let signature = signing_key.sign(account_addr.as_bytes()).unwrap();

            let code_id = abstr
                .registry()
                .module(account_module.clone())?
                .reference
                .unwrap_account()?;

            let fund_init_amount = 1_000_000u128;

            let create_msg = xion_sdk_proto::abstract_account::v1::MsgRegisterAccount {
                sender: wallet.pub_addr_str(),
                code_id,
                msg: to_json_binary(&abstract_std::account::InstantiateMsg {
                    code_id,
                    authenticator: Some(AddAuthenticator::Secp256K1 {
                        id: 1,
                        pubkey: Binary::new(signing_key.public_key().to_bytes()),
                        signature: Binary::new(signature.to_vec()),
                    }),
                    name: Some("test".to_string()),
                    account_id: Some(account_id.clone()),
                    owner: None,
                    namespace: Some("test".to_string()),
                    install_modules: vec![],
                    description: Some("foo bar".to_owned()),
                    link: None,
                })?
                .to_vec(),
                funds: vec![xion_sdk_proto::cosmos::base::v1beta1::Coin {
                    denom: "uxion".to_string(),
                    amount: fund_init_amount.to_string(),
                }],
                salt: salt.to_vec(),
            };

            xiond.rt_handle.block_on(wallet.commit_tx_any(
                vec![xionrs::Any {
                    type_url: "/abstractaccount.v1.MsgRegisterAccount".to_string(),
                    value: create_msg.encode_to_vec(),
                }],
                None,
            ))?;

            abstr.fetch_account(account_id)?
        }
    };

    println!("Account Addr: {}", account.address()?);

    // now query balance account
    let new_balance = xiond.balance(&account.address()?, Some("uxion".to_string()))?[0]
        .amount
        .u128();

    // Now attempt to burn with an account-abstracted TX
    let xion_sender = RUNTIME.block_on(xion_sender::Wallet::from_mnemonic(
        &xiond.state().chain_data,
        LOCAL_MNEMONIC,
        account.address()?.to_string(),
    ))?;

    println!("Signer configured correctly");

    let send_backmsg = xion_sdk_proto::cosmos::bank::v1beta1::MsgSend {
        from_address: account.address()?.into_string(),
        to_address: wallet.pub_addr_str(),
        amount: vec![xion_sdk_proto::cosmos::base::v1beta1::Coin {
            denom: "uxion".to_string(),
            amount: "2000".to_string(),
        }],
    };
    let exec_on_acc = xion_sdk_proto::cosmwasm::wasm::v1::MsgExecuteContract {
        sender: account.address()?.into_string(),
        contract: account.address()?.into_string(),
        msg: to_json_vec(&account::ExecuteMsg::<AddAuthenticator>::RemoveAuthMethod { id: 5 })?,
        funds: vec![],
    };
    dbg!(account.address()?.into_string());

    println!("Sending funds as signer");

    xiond.rt_handle.block_on(xion_sender.commit_tx_any(
        vec![
            xionrs::Any {
                type_url: MsgSend::type_url(),
                value: send_backmsg.encode_to_vec(),
            },
            xionrs::Any {
                type_url: xion_sdk_proto::cosmwasm::wasm::v1::MsgExecuteContract::type_url(),
                value: exec_on_acc.encode_to_vec(),
            },
        ],
        None,
    ))?;

    // now query balance account
    let newest_balance = xiond.balance(&account.address()?, Some("uxion".to_string()))?[0]
        .amount
        .u128();
    assert_ne!(newest_balance, new_balance - 2000);

    Ok(())
}

mod xion_sender {
    use xion_sdk_proto::cosmos::tx::v1beta1::{AuthInfo, SignDoc, SignerInfo, TxRaw};
    use xionrs::{
        tx::{Msg, SignMode},
        Any,
    };

    use super::*;
    /// A wallet is a sender of transactions, can be safely cloned and shared within the same thread.
    pub type Wallet = XionSender<All>;

    /// Signer of the transactions and helper for address derivation
    /// This is the main interface for simulating and signing transactions
    #[derive(Clone)]
    pub struct XionSender<C: Signing + Clone> {
        pub private_key: PrivateKey,
        pub account: String,
        /// gRPC channel
        pub grpc_channel: Channel,
        /// Information about the chain
        pub chain_info: Arc<ChainInfoOwned>,
        pub options: XionOptions,
        pub secp: Secp256k1<C>,
    }

    #[derive(Default, Clone)]
    #[non_exhaustive]
    pub struct XionOptions {
        /// Used to derive the private key
        pub key: CosmosWalletKey,
        pub account: String,
    }

    impl SenderBuilder for XionOptions {
        type Error = DaemonError;

        type Sender = Wallet;

        fn build(
            &self,
            chain_info: &Arc<ChainInfoOwned>,
        ) -> impl std::future::Future<Output = Result<Self::Sender, Self::Error>> + Send {
            XionSender::new(chain_info, self.clone())
        }
    }

    impl Wallet {
        pub async fn new(
            chain_info: &Arc<ChainInfoOwned>,
            options: XionOptions,
        ) -> Result<Wallet, DaemonError> {
            let secp = Secp256k1::new();

            let pk_from_mnemonic = |mnemonic: &str| -> Result<PrivateKey, DaemonError> {
                PrivateKey::from_words(&secp, mnemonic, 0, 0, chain_info.network_info.coin_type)
            };

            let pk: PrivateKey = match &options.key {
                CosmosWalletKey::Mnemonic(mnemonic) => pk_from_mnemonic(mnemonic)?,
                CosmosWalletKey::Env => {
                    let mnemonic = get_mnemonic_env(&chain_info.kind)?;
                    pk_from_mnemonic(&mnemonic)?
                }
                CosmosWalletKey::RawKey(bytes) => {
                    PrivateKey::from_raw_key(&secp, bytes, 0, 0, chain_info.network_info.coin_type)?
                }
            };

            // ensure address is valid
            xionrs::AccountId::new(
                &chain_info.network_info.pub_address_prefix,
                &pk.public_key(&secp).raw_address.unwrap(),
            )?;

            Ok(Self {
                account: options.account.clone(),
                chain_info: chain_info.clone(),
                grpc_channel: GrpcChannel::from_chain_info(chain_info.as_ref()).await?,
                private_key: pk,
                secp,
                options,
            })
        }

        /// Construct a new Sender from a mnemonic
        pub async fn from_mnemonic(
            chain_info: &Arc<ChainInfoOwned>,
            mnemonic: &str,
            account: String,
        ) -> Result<Wallet, DaemonError> {
            let options = XionOptions {
                key: CosmosWalletKey::Mnemonic(mnemonic.to_string()),
                account,
                ..Default::default()
            };
            Self::new(chain_info, options).await
        }

        pub fn channel(&self) -> Channel {
            self.grpc_channel.clone()
        }

        pub fn options(&self) -> XionOptions {
            self.options.clone()
        }

        /// Replaces the private key that the [XionSender] is using with key derived from the provided 24-word mnemonic.
        /// If you want more control over the derived private key, use [Self::set_private_key]
        pub fn set_mnemonic(&mut self, mnemonic: impl Into<String>) -> Result<(), DaemonError> {
            let secp = Secp256k1::new();

            let pk = PrivateKey::from_words(
                &secp,
                &mnemonic.into(),
                0,
                0,
                self.chain_info.network_info.coin_type,
            )?;
            self.set_private_key(pk);
            Ok(())
        }

        /// Replaces the private key the sender is using
        /// You can use a mnemonic to overwrite the key using [Self::set_mnemonic]
        pub fn set_private_key(&mut self, private_key: PrivateKey) {
            self.private_key = private_key
        }

        pub fn pub_addr_str(&self) -> String {
            self.account_id().to_string()
        }

        pub async fn broadcast_tx(
            &self,
            tx: TxRaw,
        ) -> Result<xion_sdk_proto::cosmos::base::abci::v1beta1::TxResponse, DaemonError> {
            let mut client =
                xion_sdk_proto::cosmos::tx::v1beta1::service_client::ServiceClient::new(
                    self.channel(),
                );
            let commit = client
                .broadcast_tx(xion_sdk_proto::cosmos::tx::v1beta1::BroadcastTxRequest {
                    tx_bytes: tx.to_bytes().unwrap(),
                    mode: xion_sdk_proto::cosmos::tx::v1beta1::BroadcastMode::Sync.into(),
                })
                .await?;
            let commit = commit.into_inner().tx_response.unwrap();

            println!("Broadcasted TX: {:#?}", commit);
            Ok(commit)
        }

        pub async fn commit_tx<T: Msg>(
            &self,
            msgs: Vec<T>,
            memo: Option<&str>,
        ) -> Result<CosmTxResponse, DaemonError> {
            let msgs = msgs
                .into_iter()
                .map(Msg::into_any)
                .collect::<Result<Vec<Any>, _>>()
                .unwrap();

            self.commit_tx_any(msgs, memo).await
        }

        pub fn sign(&self, sign_doc: SignDoc) -> Result<TxRaw, DaemonError> {
            let sign_doc_bytes = xionrs::tx::SignDoc::from(sign_doc.clone())
                .into_bytes()
                .unwrap();
            let signature = self.cosmos_private_key().sign(&sign_doc_bytes)?;

            let auth_id = auth_id::AuthId::new(1u8, true).unwrap();
            let smart_contract_sig = auth_id.signature(signature.to_vec());

            Ok(xion_sdk_proto::cosmos::tx::v1beta1::TxRaw {
                body_bytes: sign_doc.body_bytes,
                auth_info_bytes: sign_doc.auth_info_bytes,
                signatures: vec![smart_contract_sig],
            })
        }

        fn cosmos_private_key(&self) -> SigningKey {
            SigningKey::from_slice(&self.private_key.raw_key()).unwrap()
        }
    }

    impl QuerySender for Wallet {
        type Error = DaemonError;
        type Options = XionOptions;

        fn channel(&self) -> Channel {
            self.channel()
        }
    }

    impl TxSender for Wallet {
        async fn commit_tx_any(
            &self,
            msgs: Vec<Any>,
            memo: Option<&str>,
        ) -> Result<CosmTxResponse, DaemonError> {
            let timeout_height = Node::new_async(self.channel())._block_height().await? + 10u64;

            let tx_body = TxBuilder::build_body(msgs, memo, timeout_height);

            let fee = xion_sdk_proto::cosmos::tx::v1beta1::Fee {
                amount: vec![xion_sdk_proto::cosmos::base::v1beta1::Coin {
                    amount: 7500.to_string(),
                    denom: "uxion".into(),
                }],
                gas_limit: 300_000,
                payer: Default::default(),
                granter: Default::default(),
            };

            // log::debug!(
            //     target: &transaction_target(),
            //     "submitting TX: \n fee: {:?}\naccount_nr: {:?}\nsequence: {:?}",
            //     fee,
            //     account_number,
            //     sequence
            // );

            use xion_sdk_proto::cosmos::auth::v1beta1::query_client::QueryClient;

            let resp = QueryClient::new(self.channel())
                .account(QueryAccountRequest {
                    address: self.account.clone(),
                })
                .await?
                .into_inner()
                .account
                .unwrap();

            use xion_sdk_proto::abstract_account::v1::AbstractAccount;

            let account = AbstractAccount::decode(resp.value.as_ref()).unwrap();

            let account_id: cosmrs::AccountId = str::parse(account.address.as_str()).unwrap();

            let account_number = account.account_number;

            let any_pub_key = xionrs::Any {
                type_url: "/abstractaccount.v1.NilPubKey".to_string(),
                value: NilPubKey {
                    address_bytes: account_id.to_bytes(),
                }
                .encode_to_vec(),
            };

            let signer_info: SignerInfo = SignerInfo {
                public_key: Some(xionrs::tx::SignerPublicKey::Any(any_pub_key).into()),
                // public_key: self.private_key.get_signer_public_key(&self.secp),
                mode_info: Some(xionrs::tx::ModeInfo::single(SignMode::Direct).into()),
                sequence: account.sequence,
            };

            let auth_info = AuthInfo {
                signer_infos: vec![signer_info],
                fee: Some(fee),
                tip: None,
            };

            let sign_doc = SignDoc {
                body_bytes: tx_body.into_bytes().unwrap(),
                auth_info_bytes: auth_info.encode_to_vec(),
                chain_id: self.chain_info.chain_id.to_string(),
                account_number,
            };

            eprintln!("Sign doc: {:?}", sign_doc);

            let raw = self.sign(sign_doc)?;

            let resp = self.broadcast_tx(raw).await?;

            let resp = Node::new_async(self.channel())
                ._find_tx(resp.txhash)
                .await?;

            assert_broadcast_code_cosm_response(resp)
        }

        fn account_id(&self) -> cosmrs::AccountId {
            cosmrs::AccountId::new(
                &self.chain_info.network_info.pub_address_prefix,
                &self.private_key.public_key(&self.secp).raw_address.unwrap(),
            )
            // unwrap as address is validated on construction
            .unwrap()
        }
    }

    fn get_mnemonic_env(chain_kind: &ChainKind) -> Result<String, CwEnvError> {
        match chain_kind {
            ChainKind::Local => DaemonEnvVars::local_mnemonic(),
            ChainKind::Testnet => DaemonEnvVars::test_mnemonic(),
            ChainKind::Mainnet => DaemonEnvVars::main_mnemonic(),
            _ => None,
        }
        .ok_or(CwEnvError::EnvVarNotPresentNamed(
            get_mnemonic_env_name(chain_kind).to_string(),
        ))
    }

    fn get_mnemonic_env_name(chain_kind: &ChainKind) -> &str {
        match chain_kind {
            ChainKind::Local => LOCAL_MNEMONIC_ENV_NAME,
            ChainKind::Testnet => TEST_MNEMONIC_ENV_NAME,
            ChainKind::Mainnet => MAIN_MNEMONIC_ENV_NAME,
            _ => panic!("Can't set mnemonic for unspecified chainkind"),
        }
    }
}

pub mod auth_id {

    /// Authentication id for the signature
    #[cosmwasm_schema::cw_serde]
    #[derive(Copy)]
    pub struct AuthId(pub(crate) u8);

    impl AuthId {
        /// Create AuthId from signature id and flag for admin call
        /// Note: It's helper for signer, not designed to be used inside contract
        #[cfg(not(target_arch = "wasm32"))]
        pub fn new(id: u8, admin: bool) -> Option<Self> {
            let first_bit: u8 = 0b10000000;
            // If first bit occupied - we can't create AuthId
            if id & first_bit != 0 {
                return None;
            };

            Some(if admin {
                Self(id | first_bit)
            } else {
                Self(id)
            })
        }

        /// Get signature bytes with this [`AuthId`]
        /// Note: It's helper for signer, not designed to be used inside contract
        #[cfg(not(target_arch = "wasm32"))]
        pub fn signature(self, mut signature: Vec<u8>) -> Vec<u8> {
            signature.insert(0, self.0);
            signature
        }

        pub fn cred_id(self) -> (u8, bool) {
            let first_bit: u8 = 0b10000000;
            if self.0 & first_bit == 0 {
                (self.0, false)
            } else {
                (self.0 & !first_bit, true)
            }
        }
    }
}
