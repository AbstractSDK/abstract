use super::AccountId;

use cosmwasm_std::Binary;

pub const ABSTRACT_SALT: &[u8] = b"abstract";
/// Generate salt helper
pub fn generate_instantiate_salt(account_id: &AccountId) -> Binary {
    let account_id_hash = <sha2::Sha256 as sha2::Digest>::digest(account_id.to_string());
    let mut hash = account_id_hash.to_vec();
    hash.extend(ABSTRACT_SALT);
    Binary::new(hash)
}

#[cfg(test)]
mod test {
    #![allow(clippy::needless_borrows_for_generic_args)]
    use super::*;
    use crate::objects::{account::AccountTrace, TruncatedChainId};

    #[coverage_helper::test]
    fn generate_module_salt_local() {
        let salt = generate_instantiate_salt(&AccountId::local(5));
        assert!(!salt.is_empty());
        assert!(salt.len() <= 64);
    }

    #[coverage_helper::test]
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
