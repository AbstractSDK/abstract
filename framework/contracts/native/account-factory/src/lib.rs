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
    use cosmwasm_std::{testing::*, OwnedDeps};

    use crate::{contract, contract::AccountFactoryResult};

    pub fn mock_init(
        deps: &mut OwnedDeps<MockStorage, MockApi, MockQuerier>,
    ) -> AccountFactoryResult {
        let abstr = AbstractMockAddrs::new(deps.api);
        let owner = abstr.owner;
        let info = message_info(&owner, &[]);
        let admin = info.sender.to_string();

        contract::instantiate(
            deps.as_mut(),
            mock_env(),
            info,
            InstantiateMsg {
                admin,
                version_control_address: abstr.version_control.to_string(),
                ans_host_address: abstr.ans_host.to_string(),
                module_factory_address: abstr.module_factory.to_string(),
            },
        )
    }
}
