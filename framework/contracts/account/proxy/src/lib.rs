pub mod commands;
pub mod contract;
pub mod error;
mod queries;
pub mod reply;

#[cfg(test)]
mod test_common {
    use abstract_std::proxy::InstantiateMsg;
    use abstract_testing::prelude::*;
    use cosmwasm_std::{
        testing::{mock_env, mock_info, MOCK_CONTRACT_ADDR},
        DepsMut,
    };

    use crate::contract;

    pub fn mock_init(deps: DepsMut) {
        let info = mock_info(TEST_MANAGER, &[]);
        let msg = InstantiateMsg {
            account_id: TEST_ACCOUNT_ID,
            ans_host_address: MOCK_CONTRACT_ADDR.to_string(),
            manager_addr: TEST_MANAGER.to_string(),
        };
        let _res = contract::instantiate(deps, mock_env(), info, msg).unwrap();
    }
}
