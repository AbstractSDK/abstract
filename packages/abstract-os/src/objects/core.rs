use cw_storage_plus::Item;

pub type OsId = u32;

/// OS-ID storage key
pub const OS_ID: Item<OsId> = Item::new("\u{0}{5}os_id");
