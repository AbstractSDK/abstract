mod commands;
pub mod contract;
pub mod error;
mod queries;
mod validation;
mod versioning;

#[cfg(test)]
mod test_common {
    use crate::contract::ManagerResult;
    use abstract_core::{manager, objects::gov_type::GovernanceDetails};
    use abstract_testing::prelude::*;
    use cosmwasm_std::{testing::*, DepsMut};

    /// Initialize the manager with the test owner as the owner
    pub(crate) fn mock_init(mut deps: DepsMut) -> ManagerResult {
        let info = mock_info(TEST_ACCOUNT_FACTORY, &[]);

        crate::contract::instantiate(
            deps.branch(),
            mock_env(),
            info,
            manager::InstantiateMsg {
                account_id: 1,
                owner: GovernanceDetails::Monarchy {
                    monarch: TEST_OWNER.to_string(),
                },
                version_control_address: TEST_VERSION_CONTROL.to_string(),
                module_factory_address: TEST_MODULE_FACTORY.to_string(),
                name: "test".to_string(),
                description: None,
                link: None,
            },
        )
    }
}
