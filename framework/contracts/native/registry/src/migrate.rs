use abstract_std::{
    objects::module_version::assert_cw_contract_upgrade, registry::MigrateMsg, REGISTRY,
};

use cosmwasm_std::{DepsMut, Env};
use semver::Version;

use crate::contract::{VCResult, VcResponse, CONTRACT_VERSION};

#[cfg_attr(feature = "export", cosmwasm_std::entry_point)]
pub fn migrate(deps: DepsMut, env: Env, msg: MigrateMsg) -> VCResult {
    match msg {
        MigrateMsg::Instantiate(instantiate_msg) => abstract_sdk::cw_helpers::migrate_instantiate(
            deps,
            env,
            instantiate_msg,
            crate::contract::instantiate,
        ),
        MigrateMsg::Migrate {} => {
            let to_version: Version = CONTRACT_VERSION.parse()?;

            assert_cw_contract_upgrade(deps.storage, REGISTRY, to_version)?;
            cw2::set_contract_version(deps.storage, REGISTRY, CONTRACT_VERSION)?;
            Ok(VcResponse::action("migrate"))
        }
    }
}
