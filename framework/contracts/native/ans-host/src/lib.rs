pub mod commands;
pub mod contract;
pub mod error;
pub mod queries;

#[cfg(test)]
#[cfg(not(target_arch = "wasm32"))]
mod tests;

#[cfg(test)]
mod test_common {
    use abstract_std::ans_host::InstantiateMsg;
    use abstract_testing::prelude::AbstractMockAddrs;
    use cosmwasm_std::{testing::*, OwnedDeps, Response};

    use crate::{contract, error::AnsHostError};

    pub fn mock_init(
        deps: &mut OwnedDeps<MockStorage, MockApi, MockQuerier>,
    ) -> Result<Response, AnsHostError> {
        let abstr = AbstractMockAddrs::new(deps.api);
        let info = message_info(&abstr.owner, &[]);
        let admin = info.sender.to_string();

        contract::instantiate(deps.as_mut(), mock_env(), info, InstantiateMsg { admin })
    }
}
