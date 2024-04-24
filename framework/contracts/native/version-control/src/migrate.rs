use abstract_core::objects::module_version::assert_cw_contract_upgrade;
use abstract_core::version_control::state::CONFIG;
use abstract_core::version_control::Config;
use abstract_core::version_control::MigrateMsg;
use abstract_core::VERSION_CONTROL;

use cosmwasm_std::{Addr, Coin, DepsMut, Env};
use cw_semver::Version;
use cw_storage_plus::Item;

use crate::contract::{VCResult, VcResponse, CONTRACT_VERSION};

pub(crate) const CONFIG0_21: Item<Config0_21> = Item::new("cfg");

#[cosmwasm_schema::cw_serde]
pub(crate) struct Config0_21 {
    pub account_factory_address: Option<Addr>,
    pub allow_direct_module_registration_and_updates: bool,
    pub namespace_registration_fee: Option<Coin>,
}

#[cfg_attr(feature = "export", cosmwasm_std::entry_point)]
pub fn migrate(deps: DepsMut, _env: Env, _msg: MigrateMsg) -> VCResult {
    let to_version: Version = CONTRACT_VERSION.parse()?;

    let old_config = CONFIG0_21.load(deps.storage)?;
    let new_config = Config {
        account_factory_address: old_config.account_factory_address,
        security_disabled: old_config.allow_direct_module_registration_and_updates,
        namespace_registration_fee: old_config.namespace_registration_fee,
    };
    CONFIG.save(deps.storage, &new_config)?;

    assert_cw_contract_upgrade(deps.storage, VERSION_CONTROL, to_version)?;
    cw2::set_contract_version(deps.storage, VERSION_CONTROL, CONTRACT_VERSION)?;
    Ok(VcResponse::action("migrate"))
}
