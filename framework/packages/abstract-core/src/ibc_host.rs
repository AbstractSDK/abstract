//! # Abstract Api Base
//!
//! `abstract_core::adapter` implements shared functionality that's useful for creating new Abstract adapters.
//!
//! ## Description
//! An Abstract adapter contract is a contract that is allowed to perform actions on a [proxy](crate::proxy) contract.
//! It is not migratable and its functionality is shared between users, meaning that all users call the same contract address to perform operations on the Account.
//! The api structure is well-suited for implementing standard interfaces to external services like dexes, lending platforms, etc.

use crate::{
    base::{
        ExecuteMsg as MiddlewareExecMsg, InstantiateMsg as MiddlewareInstantiateMsg,
        MigrateMsg as MiddlewareMigrateMsg, QueryMsg as MiddlewareQueryMsg,
    },
    ibc_client::CallbackInfo,
    objects::account_id::AccountId,
};
use cosmwasm_schema::QueryResponses;
use cosmwasm_std::{Addr, Binary, CosmosMsg, Empty, QueryRequest};

pub type ExecuteMsg<T, R = Empty> = MiddlewareExecMsg<BaseExecuteMsg, T, R>;
pub type QueryMsg<T = Empty> = MiddlewareQueryMsg<BaseQueryMsg, T>;
pub type InstantiateMsg<T = Empty> = MiddlewareInstantiateMsg<BaseInstantiateMsg, T>;
pub type MigrateMsg<T = Empty> = MiddlewareMigrateMsg<BaseMigrateMsg, T>;

/// Used by Abstract to instantiate the contract
/// The contract is then registered on the version control contract using [`crate::version_control::ExecuteMsg::ProposeModules`].
#[cosmwasm_schema::cw_serde]
pub struct BaseInstantiateMsg {
    /// Used to easily perform address translation on the app chain
    pub ans_host_address: String,
    /// Code-id for cw1 proxy contract
    pub cw1_code_id: u64,
}

#[cosmwasm_schema::cw_serde]
pub struct BaseMigrateMsg {}

#[cosmwasm_schema::cw_serde]
pub enum InternalAction {
    Register { account_proxy_address: String },
    WhoAmI,
}

/// Callable actions on a remote host
#[cosmwasm_schema::cw_serde]
pub enum HostAction {
    App {
        msg: Binary,
    },
    Dispatch {
        msgs: Vec<CosmosMsg<Empty>>,
    },
    Query {
        msgs: Vec<QueryRequest<Empty>>,
    },
    SendAllBack {},
    Balances {},
    /// Can't be called through the packet endpoint directly
    Internal(InternalAction),
}

impl HostAction {
    pub fn into_packet(
        self,
        account_id: AccountId,
        retries: u8,
        client_chain: String,
        callback_info: Option<CallbackInfo>,
    ) -> PacketMsg {
        PacketMsg {
            client_chain,
            retries,
            callback_info,
            account_id,
            action: self,
        }
    }
}
/// This is the message we send over the IBC channel
#[cosmwasm_schema::cw_serde]
pub struct PacketMsg {
    /// Chain of the client
    pub client_chain: String,
    /// Amount of retries to attempt if packet returns with StdAck::Error
    pub retries: u8,
    pub account_id: AccountId,
    /// Callback performed after receiving an StdAck::Result
    pub callback_info: Option<CallbackInfo>,
    /// execute the custom host function
    pub action: HostAction,
}

/// Interface to the Host.
#[cosmwasm_schema::cw_serde]
pub enum BaseExecuteMsg {
    /// Update the Admin
    UpdateAdmin { admin: String },
    UpdateConfig {
        ans_host_address: Option<String>,
        cw1_code_id: Option<u64>,
    },
    /// Allow for fund recovery through the Admin
    RecoverAccount {
        closed_channel: String,
        account_id: AccountId,
        msgs: Vec<CosmosMsg>,
    },
}

/// Query Host message
#[cosmwasm_schema::cw_serde]
#[derive(QueryResponses)]
pub enum BaseQueryMsg {
    /// Returns [`HostConfigResponse`].
    #[returns(HostConfigResponse)]
    Config {},
    /// Returns (reflect) account that is attached to this channel,
    /// or none.
    #[returns(AccountResponse)]
    Account {
        client_chain: String,
        account_id: AccountId,
    },
    /// Returns all (channel, reflect_account) pairs.
    /// No pagination - this is a test contract
    #[returns(ListAccountsResponse)]
    ListAccounts {},
}

#[cosmwasm_schema::cw_serde]
pub struct HostConfigResponse {
    pub ans_host_address: Addr,
}

#[cosmwasm_schema::cw_serde]
pub struct AccountResponse {
    pub account: Option<String>,
}

#[cosmwasm_schema::cw_serde]
pub struct ListAccountsResponse {
    pub accounts: Vec<AccountInfo>,
}

#[cosmwasm_schema::cw_serde]
pub struct AccountInfo {
    pub account_id: AccountId,
    pub account: String,
    pub channel_id: String,
}

#[cfg(test)]
mod tests {
    use super::*;
    use abstract_testing::prelude::*;
    use speculoos::prelude::*;

    #[test]
    fn test_into_packet() {
        // Create an instance of HostAction (assuming a valid variant is `SomeAction`)
        let host_action = HostAction::SendAllBack {};

        // Create required parameters
        let retries = 5u8;
        let client_chain = String::from("test_client_chain");
        let callback_info = Some(CallbackInfo {
            id: "15".to_string(),
            receiver: "receiver".to_string(),
        });

        // Call into_packet function
        let packet_msg = host_action.clone().into_packet(
            TEST_ACCOUNT_ID,
            retries,
            client_chain.clone(),
            callback_info.clone(),
        );

        // Check if the returned PacketMsg has the expected values
        assert_that!(packet_msg.client_chain).is_equal_to(client_chain);
        assert_that!(packet_msg.retries).is_equal_to(retries);
        assert_that!(packet_msg.callback_info).is_equal_to(callback_info);
        assert_that!(packet_msg.account_id).is_equal_to(TEST_ACCOUNT_ID);
        assert_that!(packet_msg.action).is_equal_to(host_action);
    }
}
