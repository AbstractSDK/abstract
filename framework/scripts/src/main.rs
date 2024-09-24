use std::{env::set_var, sync::Arc};

use abstract_account::absacc::{auth::AddAuthenticator, proto::MsgRegisterAccount};
use abstract_client::{AbstractClient, Namespace};
use abstract_std::{
    objects::{module::ModuleInfo, salt::generate_instantiate_salt, AccountId},
    ACCOUNT,
};
use bitcoin::secp256k1::{All, Secp256k1, Signing};
use cosmrs::tx::Fee;
use cosmrs::{
    crypto::secp256k1::SigningKey,
    tendermint::chain::Id,
    tx::{self, ModeInfo, Msg, Raw, SignDoc, SignMode, SignerInfo},
    Any,
};
use cosmwasm_std::Addr;
use cosmwasm_std::{to_json_binary, Binary};
use cw_orch::{
    daemon::{networks::xion::XION_NETWORK, Daemon, TxSender, RUNTIME},
    prelude::*,
};
use cw_orch_core::{environment::ChainInfoOwned, CwEnvError};
use cw_orch_daemon::senders::builder::SenderBuilder;
use cw_orch_daemon::senders::CosmosWalletKey;
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
use std::str::FromStr;
use tonic::transport::Channel;
use xion_sdk_proto::abstract_account::v1::NilPubKey;
use xion_sdk_proto::cosmos::auth::v1beta1::QueryAccountRequest;
use xion_sdk_proto::traits::MessageExt;
use xion_sdk_proto::{cosmos::bank::v1beta1::MsgSend, prost::Name, traits::Message};

const GAS_BUFFER: f64 = 1.3;
const BUFFER_THRESHOLD: u64 = 200_000;
const SMALL_GAS_BUFFER: f64 = 1.4;

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
    // let abstr = AbstractClient::new(xiond.clone())
    //     .or_else(|_| AbstractClient::builder(xiond.clone()).build());
    let abstr = AbstractClient::builder(xiond.clone()).build();
    let abstr = abstr?;
    let maybe_account = abstr.account_from(Namespace::new("test")?);

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
                cosmrs::crypto::secp256k1::SigningKey::from_slice(&wallet.private_key.raw_key())
                    .unwrap();
            let signature = signing_key.sign(account_addr.as_bytes()).unwrap();

            let code_id = abstr
                .version_control()
                .module(account_module.clone())?
                .reference
                .unwrap_account()?;

            let create_msg = MsgRegisterAccount {
                sender: wallet.pub_addr_str(),
                code_id,
                msg: to_json_binary(&abstract_std::account::InstantiateMsg {
                    authenticator: Some(AddAuthenticator::Secp256K1 {
                        id: 1,
                        pubkey: Binary::new(signing_key.public_key().to_bytes()),
                        signature: Binary::new(signature.to_vec()),
                    }),
                    name: "test".to_string(),
                    account_id: Some(account_id.clone()),
                    // TODO: add new type for external Authenticator
                    owner: abstract_client::GovernanceDetails::Renounced {},
                    namespace: Some("test".to_string()),
                    install_modules: vec![],
                    description: Some("foo bar".to_owned()),
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

            abstr.account_from(account_id)?
        }
    };

    // Signature for xion account

    let auths: Vec<u8> = xiond.wasm_querier().smart_query(
        &account.address()?,
        &abstract_std::account::QueryMsg::AuthenticatorIDs {},
    )?;
    dbg!(auths);

    let old_balance = xiond.balance(&account.address()?, Some("uxion".to_string()))?[0]
        .amount
        .u128();

    // Send funds to account
    let bank_msg = xion_sdk_proto::cosmos::bank::v1beta1::MsgSend {
        from_address: wallet.pub_addr_str(),
        to_address: account.address()?.into_string(),
        amount: vec![xion_sdk_proto::cosmos::base::v1beta1::Coin {
            denom: "uxion".to_string(),
            amount: "1000".to_string(),
        }],
    };

    xiond.rt_handle.block_on(wallet.commit_tx_any(
        vec![cosmrs::Any {
            type_url: MsgSend::type_url(),
            value: bank_msg.encode_to_vec(),
        }],
        None,
    ))?;

    // now query balance account
    let new_balance = xiond.balance(&account.address()?, Some("uxion".to_string()))?[0]
        .amount
        .u128();
    assert_eq!(new_balance, old_balance + 1000);

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

    println!("Sending funds as signer");

    xiond.rt_handle.block_on(xion_sender.commit_tx_any(
        vec![cosmrs::Any {
            type_url: MsgSend::type_url(),
            value: send_backmsg.encode_to_vec(),
        }],
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
            cosmrs::AccountId::new(
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
            tx: Raw,
        ) -> Result<xion_sdk_proto::cosmos::base::abci::v1beta1::TxResponse, DaemonError> {
            let mut client =
                xion_sdk_proto::cosmos::tx::v1beta1::service_client::ServiceClient::new(
                    self.channel(),
                );
            let commit = client
                .broadcast_tx(xion_sdk_proto::cosmos::tx::v1beta1::BroadcastTxRequest {
                    tx_bytes: tx.to_bytes()?,
                    mode: xion_sdk_proto::cosmos::tx::v1beta1::BroadcastMode::Sync.into(),
                })
                .await?;

            let commit = commit.into_inner().tx_response.unwrap();
            Ok(commit)
        }

        pub async fn bank_send(
            &self,
            recipient: &Addr,
            coins: Vec<cosmwasm_std::Coin>,
        ) -> Result<CosmTxResponse, DaemonError> {
            let acc_id = self.account_id();

            let msg_send = cosmrs::bank::MsgSend {
                from_address: acc_id,
                to_address: cosmrs::AccountId::from_str(recipient.as_str())?,
                amount: parse_cw_coins(&coins)?,
            };

            self.commit_tx(vec![msg_send], Some("sending tokens")).await
        }

        /// Computes the gas needed for submitting a transaction
        pub async fn calculate_gas(
            &self,
            tx_body: &tx::Body,
            sequence: u64,
            account_number: u64,
        ) -> Result<u64, DaemonError> {
            let fee = TxBuilder::build_fee(0u8, &self.chain_info.gas_denom, 0, None)?;

            let auth_info = SignerInfo {
                public_key: self.private_key.get_signer_public_key(&self.secp),
                mode_info: ModeInfo::single(SignMode::Direct),
                sequence,
            }
            .auth_info(fee);

            let sign_doc = SignDoc::new(
                tx_body,
                &auth_info,
                &Id::try_from(self.chain_info.chain_id.to_string())?,
                account_number,
            )?;

            let tx_raw = self.sign(sign_doc)?;

            Node::new_async(self.channel())
                ._simulate_tx(tx_raw.to_bytes()?)
                .await
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

        pub fn sign(&self, sign_doc: SignDoc) -> Result<Raw, DaemonError> {
            let sign_doc_bytes = sign_doc.clone().into_bytes()?;
            let signature = self.cosmos_private_key().sign(&sign_doc_bytes)?;

            let mut smart_contract_sig = vec![1u8];
            smart_contract_sig.extend(signature.to_vec());

            Ok(Raw::from_bytes(
                &xion_sdk_proto::cosmos::tx::v1beta1::TxRaw {
                    body_bytes: sign_doc.body_bytes,
                    auth_info_bytes: sign_doc.auth_info_bytes,
                    signatures: vec![smart_contract_sig.to_vec()],
                }
                .to_bytes()
                .unwrap(),
            )
            .unwrap()
            .into())
        }

        pub(crate) fn get_fee_token(&self) -> String {
            self.chain_info.gas_denom.to_string()
        }

        fn cosmos_private_key(&self) -> SigningKey {
            SigningKey::from_slice(&self.private_key.raw_key()).unwrap()
        }

        /// Compute the gas fee from the expected gas in the transaction
        /// Applies a Gas Buffer for including signature verification
        pub(crate) fn get_fee_from_gas(&self, gas: u64) -> Result<(u64, u128), DaemonError> {
            let mut gas_expected = if let Some(gas_buffer) = DaemonEnvVars::gas_buffer() {
                gas as f64 * gas_buffer
            } else if gas < BUFFER_THRESHOLD {
                gas as f64 * SMALL_GAS_BUFFER
            } else {
                gas as f64 * GAS_BUFFER
            };

            let min_gas = DaemonEnvVars::min_gas();
            gas_expected = (min_gas as f64).max(gas_expected);

            let fee_amount = gas_expected * (self.chain_info.gas_price + 0.00001);

            Ok((gas_expected as u64, fee_amount as u128))
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

            let fee = Fee {
                amount: vec![cosmrs::Coin::new(1000_000, "uxion").unwrap()],
                gas_limit: 500_000,
                payer: None,
                granter: None,
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

            let account_number = account.account_number;

            let any_pub_key = cosmrs::Any {
                // TODO: Does it make sense to have empty type url here?
                type_url: "".to_string(),
                value: NilPubKey {
                    address_bytes: self.account.as_bytes().to_vec(),
                }
                .encode_to_vec(),
            };

            let auth_info = SignerInfo {
                public_key: Some(tx::SignerPublicKey::Any(any_pub_key)),
                // public_key: self.private_key.get_signer_public_key(&self.secp),
                mode_info: ModeInfo::single(SignMode::Direct),
                sequence: account.sequence,
            }
            .auth_info(fee);

            let sign_doc = SignDoc::new(
                &tx_body,
                &auth_info,
                &Id::try_from(self.chain_info.chain_id.to_string())?,
                account_number,
            )?;

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

    pub(crate) fn parse_cw_coins(
        coins: &[cosmwasm_std::Coin],
    ) -> Result<Vec<cosmrs::Coin>, DaemonError> {
        coins
            .iter()
            .map(|cosmwasm_std::Coin { amount, denom }| {
                Ok(cosmrs::Coin {
                    amount: amount.u128(),
                    denom: cosmrs::Denom::from_str(denom)?,
                })
            })
            .collect::<Result<Vec<_>, DaemonError>>()
    }
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
