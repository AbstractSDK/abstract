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
        testing::{message_info, mock_env, MockApi, MOCK_CONTRACT_ADDR},
        OwnedDeps,
    };

    use crate::contract;

    pub fn mock_init(deps: &mut OwnedDeps<MockStorage, MockApi, MockQuerier>) {
        let base = test_account_base(deps.api);
        let info = message_info(&base.manager, &[]);
        let msg = InstantiateMsg {
            account_id: TEST_ACCOUNT_ID,
            ans_host_address: MOCK_CONTRACT_ADDR.to_string(),
            manager_addr: base.manager.to_string(),
        };
        let _res = contract::instantiate(deps.as_mut(), mock_env(), info, msg).unwrap();
    }
}
