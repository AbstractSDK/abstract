#![cfg_attr(all(coverage_nightly, test), feature(coverage_attribute))]

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

pub mod state {
    pub use abstract_std::account::state::*;

    #[cfg(feature = "xion")]
    pub use abstract_xion::state::*;
}

/// Abstract Account
#[cfg(feature = "xion")]
pub use abstract_xion;

// re-export based on the feature
pub mod msg {
    pub use abstract_std::account::{MigrateMsg, QueryMsg};

    #[cfg(feature = "xion")]
    pub type Authenticator = crate::abstract_xion::AddAuthenticator;
    #[cfg(not(feature = "xion"))]
    pub type Authenticator = cosmwasm_std::Empty;

    pub type ExecuteMsg = abstract_std::account::ExecuteMsg<Authenticator>;
    pub type InstantiateMsg = abstract_std::account::InstantiateMsg<Authenticator>;
}

#[cfg(test)]
mod test_common {
    use abstract_std::{
        account::{self},
        objects::{account::AccountTrace, gov_type::GovernanceDetails, ownership, AccountId},
    };
    use abstract_testing::prelude::*;
    use cosmwasm_std::{testing::*, Addr, Empty, OwnedDeps};

    use crate::{contract::AccountResult, error::AccountError, msg::ExecuteMsg};

    /// Initialize the account with the test owner as the owner
    pub(crate) fn mock_init(
        deps: &mut OwnedDeps<MockStorage, MockApi, MockQuerier, Empty>,
    ) -> AccountResult {
        let abstr = AbstractMockAddrs::new(deps.api);

        let info = message_info(&abstr.owner, &[]);
        let env = mock_env_validated(deps.api);

        crate::contract::instantiate(
            deps.as_mut(),
            env,
            info,
            account::InstantiateMsg {
                code_id: 1,
                account_id: Some(AccountId::new(1, AccountTrace::Local).unwrap()),
                owner: Some(GovernanceDetails::Monarchy {
                    monarch: abstr.owner.to_string(),
                }),
                namespace: None,
                name: Some("test".to_string()),
                description: None,
                link: None,
                install_modules: vec![],
                authenticator: None,
            },
        )
    }

    pub fn test_only_owner(msg: ExecuteMsg) -> anyhow::Result<()> {
        let mut deps = mock_dependencies();
        deps.querier = abstract_mock_querier(deps.api);
        let not_owner = deps.api.addr_make("not_owner");
        mock_init(&mut deps)?;

        let res = execute_as(&mut deps, &not_owner, msg);
        assert_eq!(
            res,
            Err(AccountError::Ownership(
                ownership::GovOwnershipError::NotOwner,
            ))
        );

        Ok(())
    }

    pub fn execute_as(deps: &mut MockDeps, sender: &Addr, msg: ExecuteMsg) -> AccountResult {
        let env = mock_env_validated(deps.api);
        crate::contract::execute(deps.as_mut(), env, message_info(sender, &[]), msg)
    }

    pub fn execute_as_admin(deps: &mut MockDeps, msg: ExecuteMsg) -> AccountResult {
        let abstr = AbstractMockAddrs::new(deps.api);
        let info = message_info(&abstr.owner, &[]);
        let env = mock_env_validated(deps.api);
        crate::contract::execute(deps.as_mut(), env, info, msg)
    }
}
