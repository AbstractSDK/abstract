// TODO: make private what we can and write module-level documentation
pub mod config;
pub mod contract;
pub mod error;
pub mod execution;
pub mod migrate;
pub mod modules;
pub mod queries;
pub mod reply;
pub mod sub_account;
pub mod versioning;

#[cfg(test)]
mod test_common {
    use abstract_std::{
        account::{self, ExecuteMsg},
        objects::{account::AccountTrace, gov_type::GovernanceDetails, ownership, AccountId},
    };
    use abstract_testing::prelude::*;
    use cosmwasm_std::{testing::*, Addr, DepsMut, Empty, OwnedDeps};
    use speculoos::prelude::*;

    use crate::{contract::AccountResult, error::AccountError};

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
                account_id: Some(AccountId::new(1, AccountTrace::Local).unwrap()),
                owner: GovernanceDetails::Monarchy {
                    monarch: abstr.owner.to_string(),
                },
                namespace: None,
                name: "test".to_string(),
                description: None,
                link: None,
                install_modules: vec![],
            },
        )
    }

    pub fn test_only_owner(msg: ExecuteMsg) -> anyhow::Result<()> {
        let mut deps = mock_dependencies();
        let not_owner = deps.api.addr_make("not_owner");
        mock_init(&mut deps)?;

        let res = execute_as(deps.as_mut(), &not_owner, msg);
        assert_that!(&res)
            .is_err()
            .is_equal_to(AccountError::Ownership(
                ownership::GovOwnershipError::NotOwner,
            ));

        Ok(())
    }

    pub fn execute_as(deps: DepsMut, sender: &Addr, msg: ExecuteMsg) -> AccountResult {
        crate::contract::execute(deps, mock_env(), message_info(sender, &[]), msg)
    }

    pub fn execute_as_admin(deps: &mut MockDeps, msg: ExecuteMsg) -> AccountResult {
        let abstr = AbstractMockAddrs::new(deps.api);
        let info = message_info(&abstr.owner, &[]);
        crate::contract::execute(deps.as_mut(), mock_env(), info, msg)
    }
}
