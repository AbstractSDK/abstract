use abstract_std::{
    account::MigrateMsg, objects::module_version::assert_contract_upgrade, ACCOUNT,
};
use cosmwasm_std::{DepsMut, Env};
use cw2::set_contract_version;
use semver::Version;

use crate::contract::{AccountResponse, AccountResult, CONTRACT_VERSION};

#[cfg_attr(feature = "export", cosmwasm_std::entry_point)]
pub fn migrate(deps: DepsMut, _env: Env, _msg: MigrateMsg) -> AccountResult {
    let version: Version = CONTRACT_VERSION.parse().unwrap();

    assert_contract_upgrade(deps.storage, ACCOUNT, version)?;
    set_contract_version(deps.storage, ACCOUNT, CONTRACT_VERSION)?;
    Ok(AccountResponse::action("migrate"))
}
