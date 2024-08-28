mod commands;
pub mod contract;
pub mod error;
mod response;

pub(crate) use abstract_sdk::std::module_factory::state;

#[cfg(test)]
mod test_common {
    use abstract_testing::prelude::*;
    use cosmwasm_std::{testing::*, OwnedDeps, Response};

    use crate::{contract, error::ModuleFactoryError};

    pub fn mock_init(
        deps: &mut OwnedDeps<MockStorage, MockApi, MockQuerier>,
    ) -> Result<Response, ModuleFactoryError> {
        let abstr = AbstractMockAddrs::new(deps.api);
        let info = message_info(&abstr.owner, &[]);
        let admin = info.sender.to_string();

        contract::instantiate(
            deps.as_mut(),
            mock_env(),
            info,
            abstract_std::module_factory::InstantiateMsg {
                admin,
                version_control_address: abstr.version_control.to_string(),
                ans_host_address: abstr.ans_host.to_string(),
            },
        )
    }
}
