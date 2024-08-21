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
        DepsMut,
    };

    use crate::{contract, contract::IbcClientResult};

    pub fn mock_init(deps: DepsMut) -> IbcClientResult {
        let mock_api = MockApi::default();
        let msg = InstantiateMsg {
            ans_host_address: mock_api.addr_make(TEST_ANS_HOST).to_string(),
            version_control_address: mock_api.addr_make(TEST_VERSION_CONTROL).to_string(),
        };
        let info = message_info(&mock_api.addr_make(OWNER), &[]);
        contract::instantiate(deps, mock_env(), info, msg)
    }
}
