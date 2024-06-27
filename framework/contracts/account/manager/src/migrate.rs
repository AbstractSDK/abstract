use abstract_std::{
    manager::{
        state::{AccountInfo, CONFIG, INFO},
        MigrateMsg,
    },
    objects::module_version::assert_contract_upgrade,
    MANAGER,
};
use cosmwasm_std::{DepsMut, Env};
use cw2::set_contract_version;
use cw_gov_ownable::GovernanceDetails;
use cw_storage_plus::Item;
use semver::Version;

use crate::{
    commands::ManagerResponse,
    contract::{ManagerResult, CONTRACT_VERSION},
};

/// Abstract Account details.
#[cosmwasm_schema::cw_serde]
struct AccountInfo0_22 {
    pub name: String,
    pub governance_details: GovernanceDetails<String>,
    pub chain_id: String,
    pub description: Option<String>,
    pub link: Option<String>,
}

const INFO0_22: Item<AccountInfo0_22> = Item::new("\u{0}{4}info");

#[cfg_attr(feature = "export", cosmwasm_std::entry_point)]
pub fn migrate(mut deps: DepsMut, _env: Env, _msg: MigrateMsg) -> ManagerResult {
    // If old account info stored that includes governance details
    // We need to update account info and store new ownership
    if let Some(info) = INFO0_22.may_load(deps.storage)? {
        // Update account info
        INFO.save(
            deps.storage,
            &AccountInfo {
                name: info.name,
                chain_id: info.chain_id,
                description: info.description,
                link: info.link,
            },
        )?;
        // Update ownership
        let config = CONFIG.load(deps.storage)?;
        cw_gov_ownable::initialize_owner(
            deps.branch(),
            info.governance_details,
            config.version_control_address,
        )?;
    }

    let version: Version = CONTRACT_VERSION.parse().unwrap();

    assert_contract_upgrade(deps.storage, MANAGER, version)?;
    set_contract_version(deps.storage, MANAGER, CONTRACT_VERSION)?;
    Ok(ManagerResponse::action("migrate"))
}
