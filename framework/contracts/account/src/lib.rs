// TODO: make private what we can and write module-level documentation
pub mod actions;
pub mod config;
pub mod contract;
pub mod error;
pub mod migrate;
pub mod modules;
pub mod queries;
pub mod reply;
pub mod sub_account;
pub mod versioning;

/// Abstract Account
pub mod absacc;
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
        let abstr = AbstractMockAddrs::new(deps.api);

        let info = message_info(&abstr.account_factory, &[]);

        crate::contract::instantiate(
            deps.as_mut(),
            mock_env(),
            info,
            manager::InstantiateMsg {
                account_id: AccountId::new(1, AccountTrace::Local).unwrap(),
                owner: GovernanceDetails::Monarchy {
                    monarch: abstr.owner.to_string(),
                },
                version_control_address: abstr.version_control.to_string(),
                module_factory_address: abstr.account_factory.to_string(),
                proxy_addr: abstr.account.proxy.to_string(),
                name: "test".to_string(),
                description: None,
                link: None,
                install_modules: vec![],
            },
        )
    }
}
