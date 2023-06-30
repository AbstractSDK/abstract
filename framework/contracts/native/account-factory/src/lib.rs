mod commands;
pub mod contract;
mod error;
mod querier;
mod response;

pub(crate) use abstract_sdk::core::account_factory::state;

#[cfg(test)]
mod test_common {
    use crate::contract;
    use crate::contract::AccountFactoryResult;
    use abstract_core::account_factory::InstantiateMsg;
    use abstract_testing::prelude::*;
    use cosmwasm_std::testing::{mock_env, mock_info};
    use cosmwasm_std::DepsMut;

    pub fn mock_init(deps: DepsMut) -> AccountFactoryResult {
        contract::instantiate(
            deps,
            mock_env(),
            mock_info(TEST_ADMIN, &[]),
            InstantiateMsg {
                version_control_address: TEST_VERSION_CONTROL.to_string(),
                ans_host_address: TEST_ANS_HOST.to_string(),
                module_factory_address: TEST_MODULE_FACTORY.to_string(),
            },
        )
    }
}
