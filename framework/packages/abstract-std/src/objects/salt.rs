use super::AccountId;

use cosmwasm_std::Binary;

pub const SALT_POSTFIX: &[u8] = b"abstract";
/// Generate salt helper
pub fn generate_instantiate_salt(account_id: &AccountId) -> Binary {
    let account_id_hash = <sha2::Sha256 as sha2::Digest>::digest(account_id.to_string());
    let mut hash = account_id_hash.to_vec();
    hash.extend(SALT_POSTFIX);
    Binary(hash.to_vec())
}

#[cfg(test)]
mod test {
    #![allow(clippy::needless_borrows_for_generic_args)]
    use super::*;
    use crate::objects::{account::AccountTrace, TruncatedChainId};

    #[test]
    fn generate_module_salt_local() {
        let salt = generate_instantiate_salt(&AccountId::local(5));
        assert!(!salt.is_empty());
        assert!(salt.len() <= 64);
    }

    #[test]
    fn generate_module_salt_trace() {
        let salt = generate_instantiate_salt(
            &AccountId::new(
                5,
                AccountTrace::Remote(vec![
                    TruncatedChainId::from_chain_id("foo-1"),
                    TruncatedChainId::from_chain_id("bar-42"),
                    TruncatedChainId::from_chain_id("baz-4"),
                    TruncatedChainId::from_chain_id("qux-24"),
                    TruncatedChainId::from_chain_id("quux-99"),
                    TruncatedChainId::from_chain_id("corge-5"),
                ]),
            )
            .unwrap(),
        );
        assert!(!salt.is_empty());
        assert!(salt.len() <= 64);
    }
}
