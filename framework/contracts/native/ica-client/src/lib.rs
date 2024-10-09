pub mod contract;
pub mod error;
pub use abstract_std::ica_client as msg;
pub mod chain_types;
mod queries;

#[cfg(test)]
mod test_common {
    use crate::msg::InstantiateMsg;
    use abstract_testing::{mock_env_validated, prelude::*};
    use cosmwasm_std::{
        testing::{message_info, MockApi},
        OwnedDeps,
    };

    use crate::{contract, contract::IcaClientResult};

    pub fn mock_init(deps: &mut OwnedDeps<MockStorage, MockApi, MockQuerier>) -> IcaClientResult {
        let abstr = AbstractMockAddrs::new(deps.api);
        let msg = InstantiateMsg {};
        let info = message_info(&abstr.owner, &[]);
        let env = mock_env_validated(deps.api);

        contract::instantiate(deps.as_mut(), env, info, msg)
    }
}
