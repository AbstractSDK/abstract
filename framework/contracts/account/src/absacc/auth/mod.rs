use crate::contract::AccountResult;
use abstract_std::absacc::Authenticator;
use cosmwasm_std::{Binary, Deps, Env};
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
        id: u8,
        pubkey: Binary,
        signature: Binary,
    },
    Ed25519 {
        id: u8,
        pubkey: Binary,
        signature: Binary,
    },
    EthWallet {
        id: u8,
        address: String,
        signature: Binary,
    },
    Jwt {
        id: u8,
        aud: String,
        sub: String,
        token: Binary,
    },
    Secp256R1 {
        id: u8,
        pubkey: Binary,
        signature: Binary,
    },
    Passkey {
        id: u8,
        url: String,
        credential: Binary,
    },
}

impl AddAuthenticator {
    pub fn get_id(&self) -> u8 {
        match self {
            AddAuthenticator::Secp256K1 { id, .. } => *id,
            AddAuthenticator::Ed25519 { id, .. } => *id,
            AddAuthenticator::EthWallet { id, .. } => *id,
            AddAuthenticator::Jwt { id, .. } => *id,
            AddAuthenticator::Secp256R1 { id, .. } => *id,
            AddAuthenticator::Passkey { id, .. } => *id,
        }
    }
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
            jwt::verify(deps, &tx_bytes_hash, sig_bytes.as_slice(), aud, sub)
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
