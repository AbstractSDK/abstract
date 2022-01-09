use cosmwasm_std::{Addr, Deps, StdResult};
use cw_controllers::Admin;
use cw_storage_plus::Map;
use terraswap::asset::AssetInfo;

use dao_os::denom::is_denom;

pub const ADMIN: Admin = Admin::new("admin");


// Map with composite keys
// module name + version = code_id
// We can interate over the map giving just the prefix to get all the versions
pub const MODULE_CODE_IDS: Map<(&str, &str), u64> = Map::new("module_code_ids");

// Pair key: "ust_luna_pair"
pub const VERSION_REGISTER: Map<&str, Addr> = Map::new("version_register");

// Returns the asset info for an address book entry.
pub fn get_asset_info(deps: Deps, id: &str) -> StdResult<AssetInfo> {
    let address_or_denom = ASSET_ADDRESSES.load(deps.storage, id)?;
    return if is_denom(address_or_denom.as_str()) {
        Ok(AssetInfo::NativeToken {
            denom: address_or_denom,
        })
    } else {
        deps.api.addr_validate(address_or_denom.as_str())?;
        Ok(AssetInfo::Token {
            contract_addr: address_or_denom,
        })
    };
}
