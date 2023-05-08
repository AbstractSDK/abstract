use cw_storage_plus::Item;

pub type AccountId = u32;

/// The Account id of Abstract's admin
pub const ABSTRACT_ACCOUNT_ID: AccountId = 0;

/// Account Id storage key
pub const ACCOUNT_ID: Item<AccountId> = Item::new("\u{0}{10}account_id");
