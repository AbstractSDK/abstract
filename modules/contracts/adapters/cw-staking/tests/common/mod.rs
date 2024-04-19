#[allow(unused)]
pub const ROOT_USER: &str = "root_user";

use abstract_interface::{AbstractAccount, AccountFactory};
use abstract_std::objects::gov_type::GovernanceDetails;
use cosmwasm_std::Addr;
use cw_orch::prelude::*;

pub fn create_default_account<Chain: CwEnv>(
    factory: &AccountFactory<Chain>,
) -> anyhow::Result<AbstractAccount<Chain>> {
    let os = factory.create_default_account(GovernanceDetails::Monarchy {
        monarch: Addr::unchecked(factory.get_chain().sender()).to_string(),
    })?;
    Ok(os)
}
