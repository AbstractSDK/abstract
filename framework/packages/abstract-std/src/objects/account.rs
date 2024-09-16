mod account_id;
mod account_trace;

use cw_storage_plus::Item;

pub use self::{account_id::AccountId, account_trace::AccountTrace};

use super::common_namespace::ACCOUNT_ID_STORAGE_KEY;

pub const ABSTRACT_ACCOUNT_ID: AccountId = AccountId::const_new(0, AccountTrace::Local);
pub const TEST_ACCOUNT_ID: AccountId = AccountId::const_new(1, AccountTrace::Local);

pub type AccountSequence = u32;

/// Account Id storage key
pub const ACCOUNT_ID: Item<AccountId> = Item::new(ACCOUNT_ID_STORAGE_KEY);
