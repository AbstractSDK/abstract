use abstract_std::{
    objects::module_version::assert_cw_contract_upgrade,
    version_control::{state::CONFIG, Config, MigrateMsg},
    VERSION_CONTROL,
};

use cosmwasm_std::{DepsMut, Env};
use cw_storage_plus::Item;
use semver::Version;

use crate::contract::{VCResult, VcResponse, CONTRACT_VERSION};

#[cfg_attr(feature = "export", cosmwasm_std::entry_point)]
pub fn migrate(deps: DepsMut, _env: Env, _msg: MigrateMsg) -> VCResult {
    let to_version: Version = CONTRACT_VERSION.parse()?;

    assert_cw_contract_upgrade(deps.storage, VERSION_CONTROL, to_version)?;
    cw2::set_contract_version(deps.storage, VERSION_CONTROL, CONTRACT_VERSION)?;
    Ok(VcResponse::action("migrate"))
}
