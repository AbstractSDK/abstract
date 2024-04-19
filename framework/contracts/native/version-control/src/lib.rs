pub mod commands;
pub mod contract;
pub mod error;
pub mod queries;
#[cfg(test)]
mod testing {
    use abstract_std::version_control;
    use abstract_testing::prelude::*;
    use cosmwasm_std::{testing::*, DepsMut, Response};

    use crate::{contract, error::VCError};

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
                allow_direct_module_registration_and_updates: Some(true),
                namespace_registration_fee: None,
            },
        )
    }
}
