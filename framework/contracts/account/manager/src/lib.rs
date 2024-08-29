pub mod commands;
pub mod contract;
pub mod error;
pub(crate) mod migrate;
mod queries;
mod validation;
mod versioning;

#[cfg(test)]
mod test_common {
    use abstract_std::{
        manager,
        objects::{account::AccountTrace, gov_type::GovernanceDetails, AccountId},
    };
    use abstract_testing::prelude::*;
    use cosmwasm_std::{testing::*, DepsMut};

    use crate::contract::ManagerResult;

    /// Initialize the manager with the test owner as the owner
    pub(crate) fn mock_init(mut deps: DepsMut) -> ManagerResult {
        let info = mock_info(TEST_ACCOUNT_FACTORY, &[]);

        crate::contract::instantiate(
            deps.branch(),
            mock_env(),
            info,
            manager::InstantiateMsg {
                account_id: AccountId::new(1, AccountTrace::Local).unwrap(),
                owner: GovernanceDetails::Monarchy {
                    monarch: OWNER.to_string(),
                },
                version_control_address: TEST_VERSION_CONTROL.to_string(),
                module_factory_address: TEST_MODULE_FACTORY.to_string(),
                proxy_addr: TEST_PROXY.to_string(),
                name: "test".to_string(),
                description: None,
                link: None,
                install_modules: vec![],
            },
        )
    }
}
