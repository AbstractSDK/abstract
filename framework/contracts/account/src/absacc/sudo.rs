use cosmwasm_std::{Binary, DepsMut, Env};

use crate::{
    contract::{AccountResponse, AccountResult},
    error::AccountError,
    state::AUTHENTICATORS,
};

use super::{auth::Authenticator, AccountSudoMsg};

#[cfg_attr(feature = "export", cosmwasm_std::entry_point)]
pub fn sudo(deps: DepsMut, env: Env, msg: AccountSudoMsg) -> AccountResult {
    match msg {
        AccountSudoMsg::BeforeTx {
            tx_bytes,
            cred_bytes,
            simulate,
            msgs: _,
        } => {
            let cred_bytes = cred_bytes.ok_or(AccountError::EmptySignature {})?;
            before_tx(
                deps,
                &env,
                &Binary::from(tx_bytes.as_slice()),
                Some(Binary::from(cred_bytes.as_slice())).as_ref(),
                simulate,
            )
        }
        AccountSudoMsg::AfterTx { .. } => after_tx(),
    }
}

pub fn before_tx(
    deps: DepsMut,
    env: &Env,
    tx_bytes: &Binary,
    cred_bytes: Option<&Binary>,
    simulate: bool,
) -> AccountResult {
    if !simulate {
        let cred_bytes = cred_bytes.ok_or(AccountError::EmptySignature {})?;
        // currently, the minimum size of a signature by any auth method is 64 bytes
        // this may change in the future, and this check will need to be re-evaluated.
        //
        // checking the cred_bytes are at least 1 + 64 bytes long
        if cred_bytes.len() < 65 {
            return Err(AccountError::ShortSignature {});
        }

        // the first byte of the signature is the index of the authenticator
        let cred_index = match cred_bytes.first() {
            None => return Err(AccountError::InvalidSignature {}),
            Some(i) => *i,
        };
        crate::state::AUTH_ADMIN.save(deps.storage, &true)?;

        // retrieve the authenticator by index, or error
        let authenticator = AUTHENTICATORS.load(deps.storage, cred_index)?;

        let sig_bytes = &Binary::from(&cred_bytes.as_slice()[1..]);

        match authenticator {
            Authenticator::Secp256K1 { .. }
            | Authenticator::Ed25519 { .. }
            | Authenticator::Secp256R1 { .. } => {
                if sig_bytes.len() != 64 {
                    return Err(AccountError::ShortSignature {});
                }
            }
            Authenticator::EthWallet { .. } => {
                if sig_bytes.len() != 65 {
                    return Err(AccountError::ShortSignature {});
                }
            }
            Authenticator::Jwt { .. } => {}
            Authenticator::Passkey { .. } => {}
        }

        return match authenticator.verify(deps.as_ref(), env, tx_bytes, sig_bytes)? {
            true => Ok(AccountResponse::action("before_tx")),
            false => Err(AccountError::InvalidSignature {}),
        };
    }

    Ok(AccountResponse::action("before_tx"))
}

pub fn after_tx() -> AccountResult {
    Ok(AccountResponse::action("after_tx"))
}
