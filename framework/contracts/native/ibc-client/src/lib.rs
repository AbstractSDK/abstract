mod commands;
pub mod contract;
pub mod error;
pub mod ibc;
mod queries;

#[cfg(test)]
mod test_common {
    use abstract_std::ibc_client::InstantiateMsg;
    use abstract_testing::prelude::*;
    use cosmwasm_std::{
        testing::{message_info, mock_env, MockApi},
        OwnedDeps,
    };

    use crate::{contract, contract::IbcClientResult};

    pub fn mock_init(deps: &mut OwnedDeps<MockStorage, MockApi, MockQuerier>) -> IbcClientResult {
        let abstr = AbstractMockAddrs::new(deps.api);
        let msg = InstantiateMsg {
            ans_host_address: abstr.ans_host.to_string(),
            version_control_address: abstr.version_control.to_string(),
        };
        let info = message_info(&abstr.owner, &[]);
        contract::instantiate(deps.as_mut(), mock_env_validated(deps.api), info, msg)
    }
}
