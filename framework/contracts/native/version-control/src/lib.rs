pub mod commands;
pub mod contract;
pub mod error;
pub mod migrate;
pub mod queries;
#[cfg(test)]
mod testing {
    use abstract_std::version_control;
    use abstract_testing::prelude::*;
    use cosmwasm_std::{testing::*, DepsMut, Response};

    use crate::{
        contract,
        error::VCError,
        migrate::{Config0_21, CONFIG0_21},
    };

    /// Initialize the version_control with admin as creator and factory
    pub fn mock_init(mut deps: DepsMut) -> Result<Response, VCError> {
        let info = mock_info(OWNER, &[]);
        let admin = info.sender.to_string();

        contract::instantiate(
            deps.branch(),
            mock_env(),
            info,
            version_control::InstantiateMsg {
                admin,
                security_disabled: Some(true),
                namespace_registration_fee: None,
            },
        )
    }
    /// Initialize the version_control with admin as creator and factory
    pub fn mock_old_init(mut deps: DepsMut) -> Result<Response, VCError> {
        let init = mock_init(deps.branch())?;
        let new_config = version_control::state::CONFIG.load(deps.storage)?;
        CONFIG0_21.save(
            deps.storage,
            &Config0_21 {
                account_factory_address: new_config.account_factory_address,
                allow_direct_module_registration_and_updates: new_config.security_disabled,
                namespace_registration_fee: new_config.namespace_registration_fee,
            },
        )?;
        Ok(init)
    }
}
