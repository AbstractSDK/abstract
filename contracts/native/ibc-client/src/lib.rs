mod commands;
pub mod contract;
pub mod error;
pub mod ibc;
mod queries;

#[cfg(test)]
mod test_common {
    use crate::contract;
    use crate::contract::IbcClientResult;
    use abstract_core::ibc_client::InstantiateMsg;
    use abstract_testing::prelude::*;
    use cosmwasm_std::testing::{mock_env, mock_info};
    use cosmwasm_std::DepsMut;

    pub fn mock_init(deps: DepsMut) -> IbcClientResult {
        let msg = InstantiateMsg {
            chain: "test_chain".into(),
            ans_host_address: TEST_ANS_HOST.into(),
            version_control_address: TEST_VERSION_CONTROL.into(),
        };
        let info = mock_info(TEST_CREATOR, &[]);
        contract::instantiate(deps, mock_env(), info, msg)
    }
}
