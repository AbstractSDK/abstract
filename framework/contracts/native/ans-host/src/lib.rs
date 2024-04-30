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
    use abstract_testing::OWNER;
    use cosmwasm_std::{
        testing::{mock_env, mock_info},
        DepsMut, Response,
    };

    use crate::{contract, error::AnsHostError};

    pub fn mock_init(mut deps: DepsMut) -> Result<Response, AnsHostError> {
        let info = mock_info(OWNER, &[]);
        let admin = info.sender.to_string();

        contract::instantiate(deps.branch(), mock_env(), info, InstantiateMsg { admin })
    }
}
