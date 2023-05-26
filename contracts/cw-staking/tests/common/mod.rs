pub const ROOT_USER: &str = "root_user";

use abstract_interface::AbstractAccount;
use abstract_interface::AccountFactory;
use abstract_core::objects::gov_type::GovernanceDetails;
use cw_orch::prelude::*;
use cosmwasm_std::Addr;

pub fn create_default_account(
    factory: &AccountFactory<Mock>,
) -> anyhow::Result<AbstractAccount<Mock>> {
    let os = factory.create_default_account(GovernanceDetails::Monarchy {
        monarch: Addr::unchecked(ROOT_USER).to_string(),
    })?;
    Ok(os)
}
