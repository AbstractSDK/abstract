#![cfg_attr(all(coverage_nightly, test), feature(coverage_attribute))]

pub mod contract;
pub mod error;
pub use abstract_ica::msg;
mod chain_types;
mod queries;

#[cfg(test)]
mod test_common {
    use crate::msg::InstantiateMsg;
    use abstract_unit_test_utils::{mock_env_validated, prelude::*};
    use cosmwasm_std::{
        testing::{message_info, MockApi},
        OwnedDeps,
    };

    use crate::{contract, contract::IcaClientResult};

    pub fn mock_init(deps: &mut OwnedDeps<MockStorage, MockApi, MockQuerier>) -> IcaClientResult {
        let abstr = AbstractMockAddrs::new(deps.api);
        let msg = InstantiateMsg {
            ans_host_address: abstr.ans_host.to_string(),
            registry_address: abstr.registry.to_string(),
        };
        let info = message_info(&abstr.owner, &[]);
        let env = mock_env_validated(deps.api);

        contract::instantiate(deps.as_mut(), env, info, msg)
    }
}
