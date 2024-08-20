mod commands;
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
    use cosmwasm_std::{testing::*, Empty, OwnedDeps};

    use crate::contract::ManagerResult;

    /// Initialize the manager with the test owner as the owner
    pub(crate) fn mock_init(
        deps: &mut OwnedDeps<MockStorage, MockApi, MockQuerier, Empty>,
    ) -> ManagerResult {
        let account_factory = deps.api.addr_make(TEST_ACCOUNT_FACTORY);
        let owner = deps.api.addr_make(OWNER);
        let version_control = deps.api.addr_make(TEST_VERSION_CONTROL);
        let proxy = deps.api.addr_make(TEST_PROXY);

        let info = message_info(&account_factory, &[]);

        crate::contract::instantiate(
            deps.as_mut(),
            mock_env(),
            info,
            manager::InstantiateMsg {
                account_id: AccountId::new(1, AccountTrace::Local).unwrap(),
                owner: GovernanceDetails::Monarchy {
                    monarch: owner.to_string(),
                },
                version_control_address: version_control.to_string(),
                module_factory_address: account_factory.to_string(),
                proxy_addr: proxy.to_string(),
                name: "test".to_string(),
                description: None,
                link: None,
                install_modules: vec![],
            },
        )
    }
}
