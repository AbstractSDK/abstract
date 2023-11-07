mod commands;
pub mod contract;
mod error;
pub(crate) mod queries;
mod response;

pub(crate) use abstract_sdk::framework::account_factory::state;

#[cfg(test)]
mod test_common {
    use crate::contract;
    use crate::contract::AccountFactoryResult;
    use abstract_core::account_factory::InstantiateMsg;
    use abstract_testing::prelude::*;
    use cosmwasm_std::testing::{mock_env, mock_info};
    use cosmwasm_std::DepsMut;

    pub fn mock_init(deps: DepsMut) -> AccountFactoryResult {
        let info = mock_info(TEST_ADMIN, &[]);
        let admin = info.sender.to_string();

        contract::instantiate(
            deps,
            mock_env(),
            info,
            InstantiateMsg {
                admin,
                version_control_address: TEST_VERSION_CONTROL.to_string(),
                ans_host_address: TEST_ANS_HOST.to_string(),
                module_factory_address: TEST_MODULE_FACTORY.to_string(),
            },
        )
    }
}
