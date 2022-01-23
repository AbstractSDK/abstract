use cosmwasm_std::{Addr, Deps, StdResult};
use cw_controllers::Admin;
use cw_storage_plus::Map;
use terraswap::asset::AssetInfo;

use pandora::denom::is_denom;

pub const ADMIN: Admin = Admin::new("admin");
// stores name and address of tokens and pairs
// LP token key: "ust_luna"
pub const ASSET_ADDRESSES: Map<&str, String> = Map::new("assets");

// Pair key: "ust_luna_pair"
pub const CONTRACT_ADDRESSES: Map<&str, Addr> = Map::new("contracts");

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
