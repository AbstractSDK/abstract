mod commands;
pub mod contract;
pub mod error;
pub(crate) mod queries;
mod response;

pub(crate) use abstract_sdk::std::account_factory::state;

#[cfg(test)]
mod test_common {
    use abstract_std::account_factory::InstantiateMsg;
    use abstract_testing::prelude::*;
    use cosmwasm_std::{
        testing::{mock_env, mock_info},
        DepsMut,
    };

    use crate::{contract, contract::AccountFactoryResult};

    pub fn mock_init(deps: DepsMut) -> AccountFactoryResult {
        let info = mock_info(OWNER, &[]);
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
                verifier: None,
                min_name_length: 3,
                max_name_length: 128,
                base_price: 10u128.into(),
            },
        )
    }
}
