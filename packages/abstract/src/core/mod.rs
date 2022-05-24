pub mod manager;
pub mod modules;
pub mod proxy;

pub mod common {
    use cw_storage_plus::Item;

    pub const OS_ID: Item<u32> = Item::new("\u{0}{5}os_id");
}
