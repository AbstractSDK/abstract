#![cfg(feature = "xion")]

use abstract_account::contract::instantiate;
use abstract_std::account::InstantiateMsg;
use abstract_std::registry::state::LOCAL_ACCOUNT_SEQUENCE;
use abstract_testing::abstract_mock_querier_builder;
use abstract_testing::prelude::AbstractMockAddrs;
use abstract_xion::auth::sign_arb::wrap_message;
use abstract_xion::auth::util;
use base64::{engine::general_purpose, Engine as _};
use cosmwasm_std::testing::{message_info, mock_dependencies, mock_env};
use cosmwasm_std::{Addr, Api, Binary};

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
        authenticator: Some(abstract_xion::auth::AddAuthenticator::Secp256K1 {
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
