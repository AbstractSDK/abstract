use crate::{
    contract::{AccountResponse, AccountResult},
    error::AccountError,
};
use abstract_std::{absacc::Authenticator, account::state::AUTHENTICATOR};
use cosmwasm_std::{Binary, Deps, DepsMut, Env};
use schemars::JsonSchema;
use secp256r1::verify;
use serde::{Deserialize, Serialize};

mod eth_crypto;
pub mod jwt;
pub mod passkey;
mod secp256r1;
mod sign_arb;
pub mod util;

#[derive(Serialize, Deserialize, Clone, JsonSchema, PartialEq, Debug)]
pub enum AddAuthenticator {
    Secp256K1 {
        pubkey: Binary,
        signature: Binary,
    },
    Ed25519 {
        pubkey: Binary,
        signature: Binary,
    },
    EthWallet {
        address: String,
        signature: Binary,
    },
    Jwt {
        aud: String,
        sub: String,
        token: Binary,
    },
    Secp256R1 {
        pubkey: Binary,
        signature: Binary,
    },
    Passkey {
        url: String,
        credential: Binary,
    },
}

pub fn verify_authenticator(
    authenticator: &Authenticator,
    deps: Deps,
    env: &Env,
    tx_bytes: &Binary,
    sig_bytes: &Binary,
) -> AccountResult<bool> {
    match authenticator {
        Authenticator::Secp256K1 { pubkey } => {
            let tx_bytes_hash = util::sha256(tx_bytes);
            let verification = deps.api.secp256k1_verify(&tx_bytes_hash, sig_bytes, pubkey);
            if let Ok(ver) = verification {
                if ver {
                    return Ok(true);
                }
            }

            // if the direct verification failed, check to see if they
            // are signing with signArbitrary (common for cosmos wallets)
            let verification = sign_arb::verify(
                deps.api,
                tx_bytes.as_slice(),
                sig_bytes.as_slice(),
                pubkey.as_slice(),
            )?;
            Ok(verification)
        }
        Authenticator::Ed25519 { pubkey } => {
            let tx_bytes_hash = util::sha256(tx_bytes);
            match deps.api.ed25519_verify(&tx_bytes_hash, sig_bytes, pubkey) {
                Ok(verification) => Ok(verification),
                Err(error) => Err(error.into()),
            }
        }
        Authenticator::EthWallet { address } => {
            let addr_bytes = hex::decode(&address[2..])?;
            match eth_crypto::verify(deps.api, tx_bytes, sig_bytes, &addr_bytes) {
                Ok(_) => Ok(true),
                Err(error) => Err(error),
            }
        }
        Authenticator::Jwt { aud, sub } => {
            let tx_bytes_hash = util::sha256(tx_bytes);
            jwt::verify(
                deps,
                &Binary::from(tx_bytes_hash),
                sig_bytes.as_slice(),
                aud,
                sub,
            )
        }
        Authenticator::Secp256R1 { pubkey } => {
            let tx_bytes_hash = util::sha256(tx_bytes);
            verify(&tx_bytes_hash, sig_bytes.as_slice(), pubkey)?;

            Ok(true)
        }
        Authenticator::Passkey { url, passkey } => {
            let tx_bytes_hash = util::sha256(tx_bytes);
            passkey::verify(
                deps,
                env.clone().contract.address,
                url.clone(),
                sig_bytes,
                tx_bytes_hash,
                passkey,
            )?;

            Ok(true)
        }
    }
}

pub fn add_auth_method(
    deps: DepsMut,
    env: &Env,
    add_authenticator: AddAuthenticator,
) -> AccountResult {
    let authenticator = match add_authenticator {
        AddAuthenticator::Secp256K1 { pubkey, signature } => {
            let auth = Authenticator::Secp256K1 { pubkey };

            if !verify_authenticator(
                &auth,
                deps.as_ref(),
                &env,
                &Binary::from(env.contract.address.as_bytes()),
                &signature,
            )? {
                Err(AccountError::InvalidSignature {})
            } else {
                Ok(auth)
            }
        }
        AddAuthenticator::Ed25519 { pubkey, signature } => {
            let auth = Authenticator::Ed25519 { pubkey };

            if !verify_authenticator(
                &auth,
                deps.as_ref(),
                &env,
                &Binary::from(env.contract.address.as_bytes()),
                &signature,
            )? {
                Err(AccountError::InvalidSignature {})
            } else {
                Ok(auth)
            }
        }
        AddAuthenticator::EthWallet { address, signature } => {
            let auth = Authenticator::EthWallet { address };

            if !verify_authenticator(
                &auth,
                deps.as_ref(),
                &env,
                &Binary::from(env.contract.address.as_bytes()),
                &signature,
            )? {
                Err(AccountError::InvalidSignature {})
            } else {
                Ok(auth)
            }
        }
        AddAuthenticator::Jwt { aud, sub, token } => {
            if !jwt::verify(
                deps.as_ref(),
                &Binary::from(env.contract.address.as_bytes()),
                &token,
                &aud,
                &sub,
            )? {
                Err(AccountError::InvalidSignature {})
            } else {
                Ok(Authenticator::Jwt { aud, sub })
            }
        }
        AddAuthenticator::Secp256R1 { pubkey, signature } => {
            let auth = Authenticator::Secp256R1 { pubkey };

            if !verify_authenticator(
                &auth,
                deps.as_ref(),
                &env,
                &Binary::from(env.contract.address.as_bytes()),
                &signature,
            )? {
                Err(AccountError::InvalidSignature {})
            } else {
                Ok(auth)
            }
        }
        AddAuthenticator::Passkey { url, credential } => {
            let passkey = passkey::register(
                deps.as_ref(),
                env.contract.address.clone(),
                url.clone(),
                credential,
            )?;

            let auth = Authenticator::Passkey { url, passkey };
            Ok(auth)
        }
    }?;
    AUTHENTICATOR.save(deps.storage, &authenticator)?;
    Ok(
        AccountResponse::action("add_auth_method").add_attributes(vec![
            ("contract_address", env.contract.address.clone().to_string()),
            (
                "authenticator",
                cosmwasm_std::to_json_string(&authenticator)?,
            ),
        ]),
    )
}
