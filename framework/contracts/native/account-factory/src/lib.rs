mod commands;
pub mod contract;
pub mod error;
pub mod migrate;
pub(crate) mod queries;
mod response;

pub(crate) use abstract_sdk::std::account_factory::state;

#[cfg(test)]
mod test_common {
    use abstract_std::account_factory::InstantiateMsg;
    use abstract_testing::prelude::*;
    use cosmwasm_std::{testing::*, DepsMut};

    use crate::{contract, contract::AccountFactoryResult};

    pub fn mock_init(deps: DepsMut) -> AccountFactoryResult {
        let api = MockApi::default();
        let owner = api.addr_make(OWNER);
        let version_control_address = api.addr_make(TEST_VERSION_CONTROL);
        let ans_host_address = api.addr_make(TEST_ANS_HOST);
        let module_factory_address = api.addr_make(TEST_MODULE_FACTORY);

        let info = message_info(&owner, &[]);
        let admin = info.sender.to_string();

        contract::instantiate(
            deps,
            mock_env(),
            info,
            InstantiateMsg {
                admin,
                version_control_address: version_control_address.to_string(),
                ans_host_address: ans_host_address.to_string(),
                module_factory_address: module_factory_address.to_string(),
            },
        )
    }
}
