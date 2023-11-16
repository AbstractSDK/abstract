mod account_id;
mod account_trace;

use cw_storage_plus::Item;

pub use self::{account_id::AccountId, account_trace::AccountTrace};

pub const ABSTRACT_ACCOUNT_ID: AccountId = AccountId::const_new(0, AccountTrace::Local);
pub const TEST_ACCOUNT_ID: AccountId = AccountId::const_new(1, AccountTrace::Local);

pub type AccountSequence = u32;

/// Account Id storage key
pub const ACCOUNT_ID: Item<AccountId> = Item::new("acc_id");

/// Generate salt helper
pub fn generate_account_salt(account_id: &AccountId) -> cosmwasm_std::Binary {
    let account_id = sha256::digest(account_id.to_string());
    let salt: [u8; 32] = account_id.as_bytes()[0..32].try_into().unwrap();

    cosmwasm_std::Binary::from(salt)
}
