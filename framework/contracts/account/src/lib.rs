#![cfg_attr(all(coverage_nightly, test), feature(coverage_attribute))]

pub mod config;
pub mod contract;
pub mod error;
pub mod execution;
pub mod ics20;
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
    use abstract_std::{account::ICS20PacketIdentifier, objects::storage_namespaces};
    use cw_storage_plus::Map;

    #[cfg(feature = "xion")]
    pub type Authenticator = crate::abstract_xion::auth::AddAuthenticator;
    #[cfg(not(feature = "xion"))]
    pub type Authenticator = cosmwasm_std::Empty;

    pub type ExecuteMsg = abstract_std::account::ExecuteMsg<Authenticator>;
    pub type InstantiateMsg = abstract_std::account::InstantiateMsg<Authenticator>;

    pub const ICS20_CALLBACKS: Map<ICS20PacketIdentifier, Vec<ExecuteMsg>> =
        Map::new(storage_namespaces::account::ICS20_CALLBACKS);

    #[cosmwasm_schema::cw_serde]
    pub enum SudoMsg {
        /// For IBC hooks acknoledgments
        #[serde(rename = "ibc_lifecycle_complete")]
        IBCLifecycleComplete(IBCLifecycleComplete),
        #[cfg(feature = "xion")]
        #[serde(untagged)]
        Xion(abstract_xion::AccountSudoMsg),
    }

    #[cosmwasm_schema::cw_serde]
    pub enum IBCLifecycleComplete {
        #[serde(rename = "ibc_ack")]
        IBCAck {
            /// The source channel (osmosis side) of the IBC packet
            channel: String,
            /// The sequence number that the packet was sent with
            sequence: u64,
            /// String encoded version of the `Ack` as seen by OnAcknowledgementPacket(..)
            ack: String,
            /// Weather an `Ack` is a success of failure according to the transfer spec
            success: bool,
        },
        #[serde(rename = "ibc_timeout")]
        IBCTimeout {
            /// The source channel (osmosis side) of the IBC packet
            channel: String,
            /// The sequence number that the packet was sent with
            sequence: u64,
        },
    }
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
                account_id: Some(AccountId::new(1, AccountTrace::Local).unwrap()),
                owner: GovernanceDetails::Monarchy {
                    monarch: abstr.owner.to_string(),
                },
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
