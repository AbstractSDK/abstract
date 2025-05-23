#![cfg_attr(all(coverage_nightly, test), feature(coverage_attribute))]

mod commands;
pub mod contract;
pub mod error;

pub(crate) use abstract_sdk::std::module_factory::state;

#[cfg(test)]
mod test_common {
    use abstract_testing::{mock_env_validated, prelude::*};
    use cosmwasm_std::{testing::*, Response};

    use crate::{contract, error::ModuleFactoryError};

    pub fn mock_init(deps: &mut MockDeps) -> Result<Response, ModuleFactoryError> {
        let abstr = AbstractMockAddrs::new(deps.api);
        let info = message_info(&abstr.owner, &[]);
        let env = mock_env_validated(deps.api);
        let admin = info.sender.to_string();

        contract::instantiate(
            deps.as_mut(),
            env,
            info,
            abstract_std::module_factory::InstantiateMsg { admin },
        )
    }
}
