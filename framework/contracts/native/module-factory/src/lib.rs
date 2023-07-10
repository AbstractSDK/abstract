mod commands;
pub mod contract;
mod error;
mod querier;
mod response;

pub(crate) use abstract_sdk::core::module_factory::state;

#[cfg(test)]
mod test_common {
    use crate::contract;
    use crate::error::ModuleFactoryError;
    use abstract_testing::prelude::{TEST_ADMIN, TEST_ANS_HOST, TEST_VERSION_CONTROL};
    use cosmwasm_std::testing::*;
    use cosmwasm_std::{DepsMut, Response};

    pub fn mock_init(deps: DepsMut) -> Result<Response, ModuleFactoryError> {
        let info = mock_info(TEST_ADMIN, &[]);
        let admin = info.sender.to_string();

        contract::instantiate(
            deps,
            mock_env(),
            info,
            abstract_core::module_factory::InstantiateMsg {
                admin,
                version_control_address: TEST_VERSION_CONTROL.to_string(),
                ans_host_address: TEST_ANS_HOST.to_string(),
            },
        )
    }
}
