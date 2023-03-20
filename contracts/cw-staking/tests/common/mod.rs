pub const ROOT_USER: &str = "root_user";
use abstract_boot::OSFactory;
use abstract_boot::OS;
use abstract_os::objects::gov_type::GovernanceDetails;

use boot_core::Mock;
use cosmwasm_std::Addr;

pub fn create_default_os(factory: &OSFactory<Mock>) -> anyhow::Result<OS<Mock>> {
    let os = factory.create_default_os(GovernanceDetails::Monarchy {
        monarch: Addr::unchecked(ROOT_USER).to_string(),
    })?;
    Ok(os)
}
