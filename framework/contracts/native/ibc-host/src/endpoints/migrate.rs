use abstract_core::{objects::module_version::assert_contract_upgrade, IBC_HOST};
use abstract_sdk::core::ibc_host::MigrateMsg;
use semver::Version;

use crate::contract::{HostResponse, HostResult, CONTRACT_VERSION};

pub fn migrate(
    deps: cosmwasm_std::DepsMut,
    _env: cosmwasm_std::Env,
    _msg: MigrateMsg,
) -> HostResult {
    let to_version: Version = CONTRACT_VERSION.parse()?;

    assert_contract_upgrade(deps.storage, IBC_HOST, to_version)?;
    cw2::set_contract_version(deps.storage, IBC_HOST, CONTRACT_VERSION)?;
    Ok(HostResponse::action("migrate"))
}
