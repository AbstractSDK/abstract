mod commands;
pub mod contract;
mod error;
mod queries;
pub mod reply;

#[cfg(test)]
mod test_common {
    use abstract_core::{objects::account::TEST_ACCOUNT_ID, proxy::InstantiateMsg};
    use abstract_testing::prelude::*;
    use cosmwasm_std::{
        testing::{mock_env, mock_info},
        DepsMut,
    };

    use crate::contract;

    pub fn mock_init(deps: DepsMut) {
        let info = mock_info(TEST_MANAGER, &[]);
        let msg = InstantiateMsg {
            account_id: TEST_ACCOUNT_ID,
            manager_addr: TEST_MANAGER.to_string(),
        };
        let _res = contract::instantiate(deps, mock_env(), info, msg).unwrap();
    }
}
