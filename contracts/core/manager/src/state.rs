use cw_controllers::Admin;
use cw_storage_plus::{Item, Map};

pub const ADMIN: Admin = Admin::new("admin");
pub const OS_ID: Item<u32> = Item::new("\u{0}{5}state");
pub const OS_MODULES: Map<&str, String> = Map::new("os_modules");
