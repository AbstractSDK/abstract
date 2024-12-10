#![cfg(feature = "xion")]

use abstract_account::contract::execute;
use abstract_account::contract::instantiate;
use abstract_account::contract::AccountResult;
use abstract_account::error::AccountError;
use abstract_account::msg::ExecuteMsg;
use abstract_account::state::AUTH_ADMIN;
use abstract_std::account;
use abstract_std::account::InstantiateMsg;
use abstract_std::account::InternalConfigAction;
use abstract_std::objects::ownership::GovOwnershipError;
use abstract_std::objects::ownership::GovernanceDetails;
use abstract_std::objects::ownership::Ownership;
use abstract_std::objects::storage_namespaces::OWNERSHIP_STORAGE_KEY;
use abstract_std::objects::AccountId;
use abstract_std::objects::AccountTrace;
use abstract_std::registry::state::LOCAL_ACCOUNT_SEQUENCE;
use abstract_testing::abstract_mock_querier;
use abstract_testing::abstract_mock_querier_builder;
use abstract_testing::mock_env_validated;
use abstract_testing::prelude::AbstractMockAddrs;
use abstract_testing::prelude::*;
use abstract_xion::testing::util;
use abstract_xion::testing::wrap_message;
use base64::{engine::general_purpose, Engine as _};
use cosmwasm_std::testing::{message_info, mock_dependencies, mock_env, MockApi};
use cosmwasm_std::CosmosMsg;
use cosmwasm_std::Response;
use cosmwasm_std::{wasm_execute, Addr, Api, Binary, DepsMut, Empty, Env, OwnedDeps, WasmMsg};
use cw_storage_plus::Item;

#[test]
fn test_derive_addr() {
    let pub_key = "AxVQixKMvKkMWMgEBn5E+QjXxFLLiOUNs3EG3vvsgaGs";
    let pub_key_bytes = general_purpose::STANDARD.decode(pub_key).unwrap();

    let mut deps = mock_dependencies();
    deps.api = deps.api.with_prefix("osmo");
    let addr = util::derive_addr("osmo", pub_key_bytes.as_slice()).unwrap();

    let valid_addr = deps.api.addr_validate(addr.as_str()).unwrap();

    assert_eq!(
        "osmo1ee3y7m9kjn8xgqwryxmskv6ttnkj39z9w0fctn",
        valid_addr.as_str()
    );
}

#[test]
fn test_verify_sign_arb() {
    let pubkey = "AxVQixKMvKkMWMgEBn5E+QjXxFLLiOUNs3EG3vvsgaGs";
    let pubkey_bytes = general_purpose::STANDARD.decode(pubkey).unwrap();

    let mut deps = mock_dependencies();
    deps.api = deps.api.with_prefix("xion");
    let signer_s = util::derive_addr("xion", pubkey_bytes.as_slice()).unwrap();
    let signer = deps.api.addr_validate(signer_s.as_str()).unwrap();

    assert_eq!(
        "xion1ee3y7m9kjn8xgqwryxmskv6ttnkj39z9yaq2t2",
        signer.as_str()
    );

    let test_msg = "WooHoo";

    let test_msg_b64 = general_purpose::STANDARD.encode(test_msg);
    assert_eq!("V29vSG9v", test_msg_b64);

    let env_hash = wrap_message(test_msg.as_bytes(), signer);

    let expected_signature =
        "E5AKzlomNEYUjtYbdC8Boqlg2UIcHUL3tOq1e9CEcmlBMnONpPaAFQIZzJLIT6Jx87ViSTW58LJwGdFQqh0otA==";
    let expected_sig_bytes = general_purpose::STANDARD
        .decode(expected_signature)
        .unwrap();
    let verification = deps
        .api
        .secp256k1_verify(
            env_hash.as_slice(),
            expected_sig_bytes.as_slice(),
            pubkey_bytes.as_slice(),
        )
        .unwrap();
    assert!(verification)
}

#[test]
fn test_init_sign_arb() {
    let mut deps = mock_dependencies();
    deps.api = deps.api.with_prefix("xion");
    let abstr = AbstractMockAddrs::new(deps.api);
    deps.querier = abstract_mock_querier_builder(deps.api)
        .with_contract_item(&abstr.registry, LOCAL_ACCOUNT_SEQUENCE, &0)
        .build();
    let mut env = mock_env();
    // This is the local faucet address to simplify reuse
    env.contract.address = Addr::unchecked(
        "xion1cyyld62ly828e2xnp0c0ckpyz68wwfs26tjpscmqlaum2jcj8zdstlxvya".to_string(),
    );

    let pubkey = "Ayrlj6q3WWs91p45LVKwI8JyfMYNmWMrcDinLNEdWYE4";
    let pubkey_bytes = general_purpose::STANDARD.decode(pubkey).unwrap();

    let signer_s = util::derive_addr("xion", pubkey_bytes.as_slice()).unwrap();
    let signer = deps.api.addr_validate(signer_s.as_str()).unwrap();

    let info = message_info(&signer, &[]);

    assert_eq!(
        "xion1e2fuwe3uhq8zd9nkkk876nawrwdulgv460vzg7",
        signer.as_str()
    );

    let signature =
        "AKgG8slCFM78fE9tZzmf+L6yQskPQI0acUg3PBv/kNIO0i19i/RNaJtfFJ8A8MyHmg7Ate5imbwuzsP6mfbEaA==";
    let signature_bytes = general_purpose::STANDARD.decode(signature).unwrap();

    let instantiate_msg = InstantiateMsg {
        code_id: 1,
        authenticator: Some(abstract_xion::AddAuthenticator::Secp256K1 {
            id: 0,
            pubkey: Binary::from(pubkey_bytes),
            signature: Binary::from(signature_bytes),
        }),
        owner: abstract_std::objects::gov_type::GovernanceDetails::AbstractAccount {
            address: env.contract.address.clone(),
        },
        name: Some("account".to_owned()),
        install_modules: vec![],
        account_id: None,
        namespace: None,
        description: None,
        link: None,
    };

    instantiate(
        deps.as_mut().into_empty(),
        env.clone(),
        info,
        instantiate_msg,
    )
    .unwrap();
}

/// Initialize the account with the test owner as the owner
pub(crate) fn mock_init(
    deps: &mut OwnedDeps<MockStorage, MockApi, MockQuerier, Empty>,
) -> AccountResult {
    let abstr = AbstractMockAddrs::new(deps.api);

    let info = message_info(&abstr.owner, &[]);
    let env = mock_env_validated(deps.api);

    abstract_account::contract::instantiate(
        deps.as_mut(),
        env,
        info,
        account::InstantiateMsg {
            code_id: 1,
            account_id: Some(AccountId::new(1, AccountTrace::Local).unwrap()),
            owner: GovernanceDetails::Monarchy {
                monarch: abstr.owner.to_string(),
            },
            namespace: None,
            name: Some("test".to_string()),
            description: None,
            link: None,
            install_modules: vec![],
            authenticator: None,
        },
    )
}

const OWNERSHIP: Item<Ownership<Addr>> = Item::new(OWNERSHIP_STORAGE_KEY);

#[test]
fn xion_account_auth_itself() -> anyhow::Result<()> {
    let mut deps = mock_dependencies();
    deps.querier = abstract_mock_querier(deps.api);
    mock_init(&mut deps)?;

    let env = mock_env_validated(deps.api);
    // We set the contract as owner.
    // We can't make it through execute msgs, because of XION signatures are too messy to reproduce in tests
    let ownership = Ownership {
        owner: GovernanceDetails::AbstractAccount {
            address: env.contract.address.clone(),
        }
        .verify(deps.as_ref())?,
        pending_owner: None,
        pending_expiry: None,
    };
    OWNERSHIP.save(deps.as_mut().storage, &ownership)?;

    let whitelisted = deps.api.addr_make("whitelisted");
    let not_whitelisted_yet = deps.api.addr_make("not_whitelisted");

    // We whitelist a module
    AUTH_ADMIN.save(deps.as_mut().storage, &true)?;
    execute(
        deps.as_mut(),
        env.clone(),
        message_info(&env.contract.address, &[]),
        ExecuteMsg::UpdateInternalConfig(InternalConfigAction::UpdateWhitelist {
            to_add: vec![whitelisted.to_string()],
            to_remove: vec![],
        }),
    )?;

    // Module calls nested admin calls on account, making it admin
    let info = message_info(&whitelisted, &[]);

    let msg = ExecuteMsg::Execute {
        msgs: vec![wasm_execute(
            &env.contract.address,
            &ExecuteMsg::Execute {
                msgs: vec![wasm_execute(
                    &env.contract.address,
                    &ExecuteMsg::UpdateInternalConfig(InternalConfigAction::UpdateWhitelist {
                        to_add: vec![not_whitelisted_yet.to_string()],
                        to_remove: vec![],
                    }),
                    vec![],
                )?
                .into()],
            },
            vec![],
        )?
        .into()],
    };

    let res = execute(deps.as_mut(), env.clone(), info, msg).unwrap();

    // Execute all messages
    let res = execute_from_res(deps.as_mut(), &env, res).unwrap();
    // This should error because this action is triggered at the top by an external module
    let res = execute_from_res(deps.as_mut(), &env, res).unwrap_err();

    assert_eq!(res, AccountError::Ownership(GovOwnershipError::NotOwner));
    Ok(())
}

fn execute_from_res(deps: DepsMut, env: &Env, res: Response) -> AccountResult<Response> {
    // Execute all messages
    let info = message_info(&env.contract.address, &[]);
    if let CosmosMsg::Wasm(WasmMsg::Execute {
        contract_addr: _,
        msg,
        funds: _,
    }) = res.messages[0].msg.clone()
    {
        execute(deps, env.clone(), info, from_json(&msg)?).map_err(Into::into)
    } else {
        panic!("Wrong message received");
    }
}

mod actual_signature {
    use abstract_account::msg::InstantiateMsg;
    use abstract_account::state::{self, AUTH_ADMIN};
    use abstract_interface::{Abstract, AccountDetails, AccountExecFns, AccountI};
    use abstract_std::objects::gov_type::GovernanceDetails;
    use abstract_std::objects::salt::generate_instantiate_salt;
    use abstract_std::objects::AccountId;
    use abstract_std::registry::state::LOCAL_ACCOUNT_SEQUENCE;
    use abstract_std::ACCOUNT;
    use abstract_xion::contract::AccountSudoMsg;
    use cosmwasm_std::{to_json_binary, Binary, HexBinary};
    use cw_orch::daemon::networks::XION_TESTNET_1;
    use cw_orch::daemon::Daemon;
    use cw_orch::mock::cw_multi_test::{SudoMsg, WasmSudo};
    use cw_orch::mock::MockBech32;
    use cw_orch::prelude::{CwEnv, *};

    // From https://docs.junonetwork.io/developer-guides/junod-local-dev-setup
    pub const LOCAL_MNEMONIC: &str = "clip hire initial neck maid actor venue client foam budget lock catalog sweet steak waste crater broccoli pipe steak sister coyote moment obvious choose";
    use abstract_xion::AddAuthenticator;

    fn account_addr<Chain: CwEnv>(abstr: &Abstract<Chain>) -> anyhow::Result<Addr> {
        let chain = abstr.registry.environment().clone();

        // Generate salt from account id(or)
        let salt = generate_instantiate_salt(&AccountId::local(
            chain
                .wasm_querier()
                .item_query(&abstr.registry.address()?, LOCAL_ACCOUNT_SEQUENCE)?,
        ));
        let code_id = abstr.account_code_id().unwrap();
        let account_addr = chain
            .wasm_querier()
            .instantiate2_addr(code_id, &chain.sender_addr(), salt.clone())
            .map_err(Into::into)?;
        Ok(Addr::unchecked(account_addr))
    }

    fn xion_wallet() -> anyhow::Result<xionrs::crypto::secp256k1::SigningKey> {
        let daemon = Daemon::builder(XION_TESTNET_1)
            .mnemonic(LOCAL_MNEMONIC)
            .build()?;

        let signing_key = xionrs::crypto::secp256k1::SigningKey::from_slice(
            &daemon.sender().private_key.raw_key(),
        )
        .unwrap();

        Ok(signing_key)
    }

    fn create_xion_account<Chain: CwEnv>(
        abstr: &Abstract<Chain>,
    ) -> anyhow::Result<AccountI<Chain>> {
        let chain = abstr.registry.environment().clone();

        // Generate salt from account id(or)
        let salt = generate_instantiate_salt(&AccountId::local(
            chain
                .wasm_querier()
                .item_query(&abstr.registry.address()?, LOCAL_ACCOUNT_SEQUENCE)?,
        ));
        let code_id = abstr.account_code_id().unwrap();

        let account_addr = account_addr(&abstr)?;

        let wallet = xion_wallet()?;
        let signature = wallet.sign(account_addr.as_bytes()).unwrap();

        chain
            .instantiate2(
                code_id,
                &InstantiateMsg {
                    code_id,
                    account_id: None,
                    owner: GovernanceDetails::AbstractAccount {
                        address: account_addr.clone(),
                    },
                    namespace: None,
                    install_modules: vec![],
                    name: None,
                    description: None,
                    link: None,
                    authenticator: Some(AddAuthenticator::Secp256K1 {
                        id: 1,
                        pubkey: Binary::new(wallet.public_key().to_bytes()),
                        signature: Binary::new(signature.to_vec()),
                    }),
                },
                Some("Abstract Account"),
                Some(&account_addr),
                &[],
                salt,
            )
            .map_err(Into::into)?;

        let account_id = chain
            .wasm_querier()
            .item_query(&account_addr, state::ACCOUNT_ID)?;
        let contract_id = format!("{ACCOUNT}-{account_id}");

        let account = AccountI::new(contract_id, chain);
        account.set_address(&account_addr);
        Ok(account)
    }

    #[test]
    #[serial_test::serial]
    fn xion_account_creation() -> anyhow::Result<()> {
        let mock = MockBech32::new("xion");

        let abstr = Abstract::deploy_on(mock, ())?;

        // We create an XION abstract account
        create_xion_account(&abstr)?;

        Ok(())
    }

    fn before_hook(
        abstr: &Abstract<MockBech32>,
        account: &AccountI<MockBech32>,
    ) -> anyhow::Result<()> {
        let mock = abstr.registry.environment().clone();
        let sign_doc_bytes = Binary::from(HexBinary::from_hex("a527761bf3e9279be8cf")?);
        let signature = xion_wallet()?.sign(&sign_doc_bytes).unwrap();

        let auth_id = crate::auth_id::AuthId::new(1u8, false).unwrap();
        let smart_contract_sig = auth_id.signature(signature.to_vec());
        mock.app.borrow_mut().sudo(SudoMsg::Wasm(WasmSudo {
            contract_addr: account.address()?,
            message: to_json_binary(&AccountSudoMsg::BeforeTx {
                msgs: vec![],
                tx_bytes: sign_doc_bytes,
                cred_bytes: Some(smart_contract_sig.into()),
                simulate: false,
            })?,
        }))?;
        Ok(())
    }

    fn after_hook(
        abstr: &Abstract<MockBech32>,
        account: &AccountI<MockBech32>,
    ) -> anyhow::Result<()> {
        let mock = abstr.registry.environment().clone();

        mock.app.borrow_mut().sudo(SudoMsg::Wasm(WasmSudo {
            contract_addr: account.address()?,
            message: to_json_binary(&AccountSudoMsg::AfterTx { simulate: false })?,
        }))?;
        Ok(())
    }

    #[test]
    #[serial_test::serial]
    fn create_sub_account() -> anyhow::Result<()> {
        let mock = MockBech32::new("xion");

        let abstr = Abstract::deploy_on(mock.clone(), ())?;

        // We create an XION abstract account
        let account = create_xion_account(&abstr)?;

        before_hook(&abstr, &account)?;
        let test = mock
            .wasm_querier()
            .item_query(&account.address()?, AUTH_ADMIN)?;
        println!("{:?}", test);

        // We create a subaccount
        let sub_account = account
            .call_as(&account.address()?)
            .create_and_return_sub_account(
                AccountDetails {
                    name: "account-sub".to_string(),
                    description: None,
                    link: None,
                    namespace: None,
                    install_modules: vec![],
                    account_id: None,
                },
                &[],
            )?;

        after_hook(&abstr, &account)?;

        // The account should be able to create a sub account on the account
        // This is an admin action
        before_hook(&abstr, &account)?;
        let sub_sub_account = sub_account
            .call_as(&account.address()?)
            .create_and_return_sub_account(
                AccountDetails {
                    name: "account-sub-sub".to_string(),
                    description: None,
                    link: None,
                    namespace: None,
                    install_modules: vec![],
                    account_id: None,
                },
                &[],
            )?;
        after_hook(&abstr, &account)?;

        // The account should be able to call admin actions on the sub-sub-account
        sub_sub_account
            .call_as(&account.address()?)
            .update_status(Some(true))
            .unwrap_err();

        before_hook(&abstr, &account)?;
        sub_sub_account
            .call_as(&account.address()?)
            .update_status(Some(true))
            .unwrap();
        after_hook(&abstr, &account)?;

        Ok(())
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
