//! # Testing Functions
//! 
//! This module contains testing functions that can be used in different environments. 

pub mod manager;
// pub mod proxy; 
// pub mod account_factory;

use abstract_interface::*;
use cw_orch::prelude::*;
use abstract_sdk::core::objects::gov_type::GovernanceDetails;
use abstract_testing::prelude::*;
pub type AResult = anyhow::Result<()>; // alias for Result<(), anyhow::Error>

pub fn create_default_account<T: CwEnv>(
    factory: &AccountFactory<T>,
) -> anyhow::Result<AbstractAccount<T>> {
    let account = factory.create_default_account(GovernanceDetails::Monarchy {
        monarch: Addr::unchecked(OWNER).to_string(),
    })?;
    Ok(account)
}