use crate::AbstractXionResult;
use cosmwasm_std::{Binary, Deps, Env};
use secp256r1::verify;

pub mod eth_crypto;
pub mod jwt;
pub mod passkey;
pub mod secp256r1;
pub mod sign_arb;
pub mod util;

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

/// Note: Instead of transaction bytes address of the Abstract Account used
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
            AddAuthenticator::Secp256K1 { id, .. }
            | AddAuthenticator::Ed25519 { id, .. }
            | AddAuthenticator::EthWallet { id, .. }
            | AddAuthenticator::Jwt { id, .. }
            | AddAuthenticator::Secp256R1 { id, .. }
            | AddAuthenticator::Passkey { id, .. } => *id,
        }
    }
}

#[cosmwasm_schema::cw_serde]
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
    ) -> AbstractXionResult<bool> {
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
}

pub mod execute {
    use super::*;

    use crate::{error::AbstractXionError, state::AUTHENTICATORS, AbstractXionResult};
    use cosmwasm_std::{Binary, DepsMut, Env, Event, Order, Response};

    pub fn add_auth_method(
        deps: DepsMut,
        env: &Env,
        add_authenticator: &mut AddAuthenticator,
    ) -> AbstractXionResult {
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
                    env,
                    &Binary::from(env.contract.address.as_bytes()),
                    signature,
                )? {
                    Err(AbstractXionError::InvalidSignature {})
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
                    env,
                    &Binary::from(env.contract.address.as_bytes()),
                    signature,
                )? {
                    Err(AbstractXionError::InvalidSignature {})
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
                    env,
                    &Binary::from(env.contract.address.as_bytes()),
                    signature,
                )? {
                    Err(AbstractXionError::InvalidSignature {})
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
                    env,
                    &Binary::from(env.contract.address.as_bytes()),
                    signature,
                )? {
                    Err(AbstractXionError::InvalidSignature {})
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

    pub fn remove_auth_method(deps: DepsMut, env: Env, id: u8) -> AbstractXionResult {
        if AUTHENTICATORS
            .keys(deps.storage, None, None, Order::Ascending)
            .count()
            <= 1
        {
            return Err(AbstractXionError::MinimumAuthenticatorCount {});
        }

        AUTHENTICATORS.remove(deps.storage, id);
        Ok(
            Response::new().add_event(Event::new("remove_auth_method").add_attributes(vec![
                ("contract_address", env.contract.address.to_string()),
                ("authenticator_id", id.to_string()),
            ])),
        )
    }

    fn save_authenticator(
        deps: DepsMut,
        id: u8,
        authenticator: &Authenticator,
    ) -> AbstractXionResult<()> {
        // TODO: recover check after discussion with xion
        // if id > 127 {
        //     return Err(AbstractXionError::TooBigAuthId {});
        // }
        if AUTHENTICATORS.has(deps.storage, id) {
            return Err(AbstractXionError::OverridingIndex { index: id });
        }

        AUTHENTICATORS.save(deps.storage, id, authenticator)?;
        Ok(())
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[coverage_helper::test]
    fn auth_id_err() {
        for id in 128..=u8::MAX {
            assert!(AuthId::new(id, true).is_none())
        }
    }

    #[coverage_helper::test]
    fn auth_id() {
        for id in 0..0b10000000 {
            let (unmasked_id, admin) = AuthId::new(id, true).unwrap().cred_id();
            assert_eq!(id, unmasked_id);
            assert!(admin);

            let (unmasked_id, admin) = AuthId::new(id, false).unwrap().cred_id();
            assert_eq!(id, unmasked_id);
            assert!(!admin);
        }
    }
}
