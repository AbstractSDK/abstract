use abstract_std::{
    account_factory::{
        state::{Config, CONFIG},
        MigrateMsg,
    },
    objects::module_version::assert_contract_upgrade,
    ACCOUNT_FACTORY,
};
use cosmwasm_std::{Addr, DepsMut, Env};
use cw_storage_plus::Item;
use semver::Version;

use crate::contract::{AccountFactoryResponse, AccountFactoryResult, CONTRACT_VERSION};

pub(crate) const CONFIG0_22: Item<Config0_22> = Item::new("cfg");

#[cosmwasm_schema::cw_serde]
pub(crate) struct Config0_22 {
    pub version_control_contract: Addr,
    pub ans_host_contract: Addr,
    pub module_factory_address: Addr,
    pub ibc_host: Option<Addr>,
}

#[cfg_attr(feature = "export", cosmwasm_std::entry_point)]
pub fn migrate(deps: DepsMut, _env: Env, _msg: MigrateMsg) -> AccountFactoryResult {
    let version: Version = CONTRACT_VERSION.parse().unwrap();

    assert_contract_upgrade(deps.storage, ACCOUNT_FACTORY, version)?;
    cw2::set_contract_version(deps.storage, ACCOUNT_FACTORY, CONTRACT_VERSION)?;

    if let Ok(old_config) = CONFIG0_22.load(deps.storage) {
        let new_config = Config {
            version_control_contract: old_config.version_control_contract,
            ans_host_contract: old_config.ans_host_contract,
            module_factory_address: old_config.module_factory_address,
        };
        // No need to remove old config, because this uses same storage key
        CONFIG.save(deps.storage, &new_config)?;
    }

    Ok(AccountFactoryResponse::action("migrate"))
}
