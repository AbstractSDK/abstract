//! # Abstract Api Base
//!
//! `abstract_os::api` implements shared functionality that's useful for creating new Abstract apis.
//!
//! ## Description
//! An Abstract api contract is a contract that is allowed to perform actions on a [proxy](crate::proxy) contract.
//! It is not migratable and its functionality is shared between users, meaning that all users call the same contract address to perform operations on the OS.
//! The api structure is well-suited for implementing standard interfaces to external services like dexes, lending platforms, etc.

use crate::base::{
    ExecuteMsg as MiddlewareExecMsg, InstantiateMsg as MiddlewareInstantiateMsg,
    MigrateMsg as MiddlewareMigrateMsg, QueryMsg as MiddlewareQueryMsg,
};
use crate::ibc_client::CallbackInfo;
use cosmwasm_schema::QueryResponses;
use cosmwasm_std::{Addr, Binary, CosmosMsg, Empty, QueryRequest};
use crate::objects::core::OsId;

pub type ExecuteMsg<T, R = Empty> = MiddlewareExecMsg<BaseExecuteMsg, T, R>;
pub type QueryMsg<T = Empty> = MiddlewareQueryMsg<BaseQueryMsg, T>;
pub type InstantiateMsg<T = Empty> = MiddlewareInstantiateMsg<BaseInstantiateMsg, T>;
pub type MigrateMsg<T = Empty> = MiddlewareMigrateMsg<BaseMigrateMsg, T>;

/// Used by Abstract to instantiate the contract
/// The contract is then registered on the version control contract using [`crate::version_control::ExecuteMsg::AddApi`].
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
    Register { os_proxy_address: String },
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
        os_id: OsId,
        retries: u8,
        client_chain: String,
        callback_info: Option<CallbackInfo>,
    ) -> PacketMsg {
        PacketMsg {
            client_chain,
            retries,
            callback_info,
            os_id,
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
    pub os_id: OsId,
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
        os_id: OsId,
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
    Account { client_chain: String, os_id: OsId },
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
    pub os_id: OsId,
    pub account: String,
    pub channel_id: String,
}
