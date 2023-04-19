mod commands;
pub mod contract;
mod error;
mod queries;

#[cfg(test)]
mod test_common {
    use crate::contract;
    use abstract_core::proxy::InstantiateMsg;
    use abstract_testing::prelude::*;
    use cosmwasm_std::testing::{mock_env, mock_info, MOCK_CONTRACT_ADDR};
    use cosmwasm_std::DepsMut;

    pub fn mock_init(deps: DepsMut) {
        let info = mock_info(TEST_MANAGER, &[]);
        let msg = InstantiateMsg {
            account_id: 0,
            ans_host_address: MOCK_CONTRACT_ADDR.to_string(),
        };
        let _res = contract::instantiate(deps, mock_env(), info, msg).unwrap();
    }
}
