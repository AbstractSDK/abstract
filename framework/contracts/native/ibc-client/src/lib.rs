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
        testing::{mock_env, mock_info},
        DepsMut,
    };

    use crate::{contract, contract::IbcClientResult};

    pub fn mock_init(deps: DepsMut) -> IbcClientResult {
        let msg = InstantiateMsg {
            ans_host_address: TEST_ANS_HOST.into(),
            version_control_address: TEST_VERSION_CONTROL.into(),
        };
        let info = mock_info(OWNER, &[]);
        contract::instantiate(deps, mock_env(), info, msg)
    }
}
