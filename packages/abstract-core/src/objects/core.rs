use cw_storage_plus::Item;

pub type AccountId = u32;

/// Account Id storage key
pub const ACCOUNT_ID: Item<AccountId> = Item::new("\u{0}{10}account_id");
