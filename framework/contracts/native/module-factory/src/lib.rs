mod commands;
pub mod contract;
pub mod error;
mod response;

pub(crate) use abstract_sdk::std::module_factory::state;

#[cfg(test)]
mod test_common {
    use abstract_testing::prelude::*;
    use cosmwasm_std::{testing::*, DepsMut, Response};

    use crate::{contract, error::ModuleFactoryError};

    pub fn mock_init(deps: DepsMut) -> Result<Response, ModuleFactoryError> {
        let info = mock_info(OWNER, &[]);
        let admin = info.sender.to_string();

        contract::instantiate(
            deps,
            mock_env(),
            info,
            abstract_std::module_factory::InstantiateMsg {
                admin,
                version_control_address: TEST_VERSION_CONTROL.to_string(),
                ans_host_address: TEST_ANS_HOST.to_string(),
            },
        )
    }
}
