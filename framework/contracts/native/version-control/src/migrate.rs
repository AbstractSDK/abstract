use abstract_std::{
    objects::module_version::assert_cw_contract_upgrade,
    version_control::{state::CONFIG, Config, MigrateMsg},
    VERSION_CONTROL,
};

use cosmwasm_std::{DepsMut, Env, MessageInfo};
use cw_storage_plus::Item;
use semver::Version;

use crate::contract::{VCResult, VcResponse, CONTRACT_VERSION};

#[cfg_attr(feature = "export", cosmwasm_std::entry_point)]
pub fn migrate(deps: DepsMut, env: Env, msg: MigrateMsg) -> VCResult {
    match msg {
        MigrateMsg::Instantiate(instantiate_msg) => {
            let contract_info = deps
                .querier
                .query_wasm_contract_info(&env.contract.address)?;
            // Only admin can call migrate on contract
            let sender = contract_info.admin.unwrap();
            let message_info = MessageInfo {
                sender,
                funds: vec![],
            };
            crate::contract::instantiate(deps, env, message_info, instantiate_msg)
        }
        MigrateMsg::Migrate {} => {
            let to_version: Version = CONTRACT_VERSION.parse()?;

            assert_cw_contract_upgrade(deps.storage, VERSION_CONTROL, to_version)?;
            cw2::set_contract_version(deps.storage, VERSION_CONTROL, CONTRACT_VERSION)?;
            Ok(VcResponse::action("migrate"))
        }
    }
}
