use cw_controllers::Admin;
use cw_storage_plus::Map;

pub const ADMIN: Admin = Admin::new("admin");

// Map with composite keys
// module name + version = code_id
// We can interate over the map giving just the prefix to get all the versions
pub const MODULE_CODE_IDS: Map<(&str, &str), u64> = Map::new("module_code_ids");