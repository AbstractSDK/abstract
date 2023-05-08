pub mod commands;
pub mod contract;
pub mod error;
pub mod queries;

#[cfg(test)]
mod testing {
    use crate::contract;
    use crate::error::VCError;
    use abstract_core::version_control;
    use abstract_testing::prelude::*;
    use cosmwasm_std::testing::*;
    use cosmwasm_std::DepsMut;
    use cosmwasm_std::Response;

    /// Initialize the version_control with admin as creator and factory
    pub fn mock_init(mut deps: DepsMut) -> Result<Response, VCError> {
        let info = mock_info(TEST_ADMIN, &[]);
        contract::instantiate(
            deps.branch(),
            mock_env(),
            info,
            version_control::InstantiateMsg {
                is_testnet: true,
                namespace_limit: 10,
            },
        )
    }
}
