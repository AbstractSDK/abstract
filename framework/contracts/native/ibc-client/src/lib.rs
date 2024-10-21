#![cfg_attr(all(coverage_nightly, test), feature(coverage_attribute))]

mod anybuf;
mod commands;
pub mod contract;
pub mod error;
pub mod ibc;
mod queries;

#[cfg(test)]
mod test_common {
    use abstract_std::ibc_client::InstantiateMsg;
    use abstract_testing::{mock_env_validated, prelude::*};
    use cosmwasm_std::{
        testing::{message_info, MockApi},
        OwnedDeps,
    };

    use crate::{contract, contract::IbcClientResult};

    pub fn mock_init(deps: &mut OwnedDeps<MockStorage, MockApi, MockQuerier>) -> IbcClientResult {
        let abstr = AbstractMockAddrs::new(deps.api);
        let msg = InstantiateMsg {};
        let info = message_info(&abstr.owner, &[]);
        let env = mock_env_validated(deps.api);
        contract::instantiate(deps.as_mut(), env, info, msg)
    }
}
