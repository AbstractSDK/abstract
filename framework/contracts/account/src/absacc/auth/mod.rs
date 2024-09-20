use crate::{
    contract::{AccountResponse, AccountResult},
    error::AccountError,
    state::AUTHENTICATORS,
};
use cosmwasm_std::{Binary, Deps, DepsMut, Env, Event, Response};
use schemars::JsonSchema;
use secp256r1::verify;
use serde::{Deserialize, Serialize};

mod eth_crypto;
pub mod jwt;
pub mod passkey;
mod secp256r1;
mod sign_arb;
pub mod util;

#[cosmwasm_schema::cw_serde]
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

#[derive(Serialize, Deserialize, Clone, JsonSchema, PartialEq, Debug)]
pub enum Authenticator {
    Secp256K1 { pubkey: Binary },
    Ed25519 { pubkey: Binary },
    EthWallet { address: String },
    Jwt { aud: String, sub: String },
    Secp256R1 { pubkey: Binary },
    Passkey { url: String, passkey: Binary },
}

impl Authenticator {
    pub fn verify(
        &self,
        deps: Deps,
        env: &Env,
        tx_bytes: &Binary,
        sig_bytes: &Binary,
    ) -> AccountResult<bool> {
        match self {
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
        add_authenticator: &mut AddAuthenticator,
    ) -> AccountResult {
        match add_authenticator {
            AddAuthenticator::Secp256K1 {
                id,
                pubkey,
                signature,
            } => {
                let auth = Authenticator::Secp256K1 {
                    pubkey: pubkey.clone(),
                };

                if !auth.verify(
                    deps.as_ref(),
                    &env,
                    &Binary::from(env.contract.address.as_bytes()),
                    signature,
                )? {
                    Err(AccountError::InvalidSignature {})
                } else {
                    save_authenticator(deps, *id, &auth)?;
                    Ok(())
                }
            }
            AddAuthenticator::Ed25519 {
                id,
                pubkey,
                signature,
            } => {
                let auth = Authenticator::Ed25519 {
                    pubkey: pubkey.clone(),
                };

                if !auth.verify(
                    deps.as_ref(),
                    &env,
                    &Binary::from(env.contract.address.as_bytes()),
                    signature,
                )? {
                    Err(AccountError::InvalidSignature {})
                } else {
                    save_authenticator(deps, *id, &auth)?;
                    Ok(())
                }
            }
            AddAuthenticator::EthWallet {
                id,
                address,
                signature,
            } => {
                let auth = Authenticator::EthWallet {
                    address: address.clone(),
                };

                if !auth.verify(
                    deps.as_ref(),
                    &env,
                    &Binary::from(env.contract.address.as_bytes()),
                    signature,
                )? {
                    Err(AccountError::InvalidSignature {})
                } else {
                    save_authenticator(deps, *id, &auth)?;
                    Ok(())
                }
            }
            AddAuthenticator::Jwt {
                id,
                aud,
                sub,
                token,
            } => {
                let auth = Authenticator::Jwt {
                    aud: aud.clone(),
                    sub: sub.clone(),
                };

                jwt::verify(
                    deps.as_ref(),
                    &Binary::from(env.contract.address.as_bytes()),
                    token,
                    aud,
                    sub,
                )?;

                save_authenticator(deps, *id, &auth)?;
                Ok(())
            }
            AddAuthenticator::Secp256R1 {
                id,
                pubkey,
                signature,
            } => {
                let auth = Authenticator::Secp256R1 {
                    pubkey: pubkey.clone(),
                };

                if !auth.verify(
                    deps.as_ref(),
                    &env,
                    &Binary::from(env.contract.address.as_bytes()),
                    signature,
                )? {
                    Err(AccountError::InvalidSignature {})
                } else {
                    AUTHENTICATORS.save(deps.storage, *id, &auth)?;
                    Ok(())
                }
            }
            AddAuthenticator::Passkey {
                id,
                url,
                credential,
            } => {
                let passkey = passkey::register(
                    deps.as_ref(),
                    env.contract.address.clone(),
                    (*url).clone(),
                    (*credential).clone(),
                )?;

                let auth = Authenticator::Passkey {
                    url: (*url).clone(),
                    passkey: passkey.clone(),
                };
                save_authenticator(deps, *id, &auth)?;
                // we replace the sent credential with the passkey for indexers and other
                // observers to see
                *(credential) = passkey;
                Ok(())
            }
        }?;
        Ok(
            Response::new().add_event(Event::new("add_auth_method").add_attributes(vec![
                ("contract_address", env.contract.address.clone().to_string()),
                (
                    "authenticator",
                    cosmwasm_std::to_json_string(&add_authenticator)?,
                ),
            ])),
        )
    }
}

pub fn save_authenticator(
    deps: DepsMut,
    id: u8,
    authenticator: &Authenticator,
) -> AccountResult<()> {
    if AUTHENTICATORS.has(deps.storage, id) {
        return Err(AccountError::OverridingIndex { index: id });
    }

    AUTHENTICATORS.save(deps.storage, id, authenticator)?;
    Ok(())
}
