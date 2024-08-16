use abstract_std::{
    manager::MigrateMsg, objects::module_version::assert_contract_upgrade, MANAGER,
};
use cosmwasm_std::{DepsMut, Env};
use cw2::set_contract_version;
use semver::Version;

use crate::{
    commands::ManagerResponse,
    contract::{ManagerResult, CONTRACT_VERSION},
};

#[cfg_attr(feature = "export", cosmwasm_std::entry_point)]
pub fn migrate(deps: DepsMut, _env: Env, _msg: MigrateMsg) -> ManagerResult {
    let version: Version = CONTRACT_VERSION.parse().unwrap();

    assert_contract_upgrade(deps.storage, MANAGER, version)?;
    set_contract_version(deps.storage, MANAGER, CONTRACT_VERSION)?;
    Ok(ManagerResponse::action("migrate"))
}
