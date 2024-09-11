pub mod contract;
pub mod error;
pub mod migrate;

#[cfg(test)]
mod test_common {
    use abstract_std::{
        account,
        objects::{account::AccountTrace, gov_type::GovernanceDetails, AccountId},
    };
    use abstract_testing::prelude::*;
    use cosmwasm_std::{testing::*, Empty, OwnedDeps};

    use crate::contract::AccountResult;

    /// Initialize the manager with the test owner as the owner
    pub(crate) fn mock_init(
        deps: &mut OwnedDeps<MockStorage, MockApi, MockQuerier, Empty>,
    ) -> AccountResult {
        let abstr = AbstractMockAddrs::new(deps.api);
        let info = message_info(&abstr.owner, &[]);

        crate::contract::instantiate(
            deps.as_mut(),
            mock_env(),
            info,
            account::InstantiateMsg {
                account_id: AccountId::new(1, AccountTrace::Local).ok(),
                owner: GovernanceDetails::Monarchy {
                    monarch: abstr.owner.to_string(),
                },
                version_control_address: abstr.version_control.to_string(),
                module_factory_address: abstr.module_factory.to_string(),
                namespace: None,
                name: "test".to_string(),
                description: None,
                link: None,
                install_modules: vec![],
            },
        )
    }
}
