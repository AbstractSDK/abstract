pub mod contract;
pub mod error;
pub use abstract_ica::msg;
mod chain_types;
mod queries;
pub(crate) mod state;

#[cfg(test)]
mod test_common {
    use crate::msg::InstantiateMsg;
    use abstract_testing::prelude::*;
    use cosmwasm_std::{
        testing::{message_info, mock_env, MockApi},
        OwnedDeps,
    };

    use crate::{contract, contract::IcaClientResult};

    pub fn mock_init(deps: &mut OwnedDeps<MockStorage, MockApi, MockQuerier>) -> IcaClientResult {
        let abstr = AbstractMockAddrs::new(deps.api);
        let msg = InstantiateMsg {
            ans_host_address: abstr.ans_host.to_string(),
            version_control_address: abstr.version_control.to_string(),
        };
        let info = message_info(&abstr.owner, &[]);
        contract::instantiate(deps.as_mut(), mock_env(), info, msg)
    }
}
