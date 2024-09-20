use crate::contract::AccountResult;

use super::util::{self, derive_addr, sha256};
use base64::{engine::general_purpose, Engine as _};
use cosmwasm_std::{Addr, Api};

pub fn verify(
    api: &dyn Api,
    msg_bytes: &[u8],
    sig_bytes: &[u8],
    pubkey: &[u8],
) -> AccountResult<bool> {
    let signer_s = derive_addr(util::CHAIN_BECH_PREFIX, pubkey)?;
    let signer = api.addr_validate(signer_s.as_str())?;

    let envelope_hash = wrap_message(msg_bytes, signer);

    let verification = api.secp256k1_verify(envelope_hash.as_slice(), sig_bytes, pubkey)?;
    Ok(verification)
}

fn wrap_message(msg_bytes: &[u8], signer: Addr) -> Vec<u8> {
    let msg_b64 = general_purpose::STANDARD.encode(msg_bytes);
    // format the msg in the style of ADR-036 SignArbitrary
    let envelope = format!("{{\"account_number\":\"0\",\"chain_id\":\"\",\"fee\":{{\"amount\":[],\"gas\":\"0\"}},\"memo\":\"\",\"msgs\":[{{\"type\":\"sign/MsgSignData\",\"value\":{{\"data\":\"{}\",\"signer\":\"{}\"}}}}],\"sequence\":\"0\"}}", msg_b64.as_str(), signer.as_str());

    sha256(envelope.to_string().as_bytes())
}

#[cfg(test)]
mod tests {
    use super::super::sign_arb::wrap_message;
    use super::super::util;
    use crate::contract::instantiate;
    use abstract_std::account::InstantiateMsg;
    use base64::{engine::general_purpose, Engine as _};
    use cosmwasm_std::testing::{
        message_info, mock_dependencies, mock_env, MockApi, MockQuerier, MockStorage,
    };
    use cosmwasm_std::{Addr, Api, Binary, OwnedDeps};

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

        let expected_signature = "E5AKzlomNEYUjtYbdC8Boqlg2UIcHUL3tOq1e9CEcmlBMnONpPaAFQIZzJLIT6Jx87ViSTW58LJwGdFQqh0otA==";
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
        let mut deps = OwnedDeps {
            storage: MockStorage::default(),
            api: MockApi::default().with_prefix("xion"),
            querier: MockQuerier::<XionCustomQuery>::new(&[]),
            custom_query_type: core::marker::PhantomData::<XionCustomQuery>,
        };
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

        let signature = "AKgG8slCFM78fE9tZzmf+L6yQskPQI0acUg3PBv/kNIO0i19i/RNaJtfFJ8A8MyHmg7Ate5imbwuzsP6mfbEaA==";
        let signature_bytes = general_purpose::STANDARD.decode(signature).unwrap();

        // TODO:
        // let instantiate_msg = InstantiateMsg {
        //     authenticator: crate::auth::AddAuthenticator::Secp256K1 {
        //         id: 0,
        //         pubkey: Binary::from(pubkey_bytes),
        //         signature: Binary::from(signature_bytes),
        //     },
        // };

        // instantiate(
        //     deps.as_mut().into_empty(),
        //     env.clone(),
        //     info,
        //     instantiate_msg,
        // )
        // .unwrap();
    }
}
