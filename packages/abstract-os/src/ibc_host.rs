//! # Abstract API Base
//!
//! `abstract_os::api` implements shared functionality that's useful for creating new Abstract APIs.
//!
//! ## Description
//! An Abstract API contract is a contract that is allowed to perform actions on a [proxy](crate::proxy) contract.
//! It is not migratable and its functionality is shared between users, meaning that all users call the same contract address to perform operations on the OS.
//! The API structure is well-suited for implementing standard interfaces to external services like dexes, lending platforms, etc.

use cosmwasm_schema::QueryResponses;
use cosmwasm_std::{Addr, Binary, CosmosMsg, Empty, QueryRequest};
use serde::Serialize;

use crate::ibc_client::CallbackInfo;

/// Used by Abstract to instantiate the contract
/// The contract is then registered on the version control contract using [`crate::version_control::ExecuteMsg::AddApi`].
#[cosmwasm_schema::cw_serde]
pub struct BaseInstantiateMsg {
    /// Used to easily perform address translation on the app chain
    pub memory_address: String,
    /// Code-id for cw1 proxy contract
    pub cw1_code_id: u64,
}

#[cosmwasm_schema::cw_serde]
pub struct MigrateMsg {}

#[cosmwasm_schema::cw_serde]
pub enum InternalAction {
    Register,
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
    /// Fill with [`Option::None`] on call. Gets filled by IBC client.
    SendAllBack {
        os_proxy_address: Option<String>,
    },
    Balances {},
    /// Can't be called through the packet endpoint directly
    Internal(InternalAction),
}

impl HostAction {
    pub fn into_packet(
        self,
        os_id: u32,
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
    pub os_id: u32,
    /// Callback performed after receiving an StdAck::Result
    pub callback_info: Option<CallbackInfo>,
    /// execute the custom host function
    pub action: HostAction,
}

/// Interface to the Host.
#[cosmwasm_schema::cw_serde]
pub enum ExecuteMsg {
    ClearAccount { closed_channel: String, os_id: u32 },
}

#[cosmwasm_schema::cw_serde]
pub enum QueryMsg<Q: Serialize = Empty> {
    App(Q),
    /// A configuration message to whitelist traders.
    Base(BaseQueryMsg),
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
    Account { client_chain: String, os_id: u32 },
    /// Returns all (channel, reflect_account) pairs.
    /// No pagination - this is a test contract
    #[returns(ListAccountsResponse)]
    ListAccounts {},
}

#[cosmwasm_schema::cw_serde]
pub struct HostConfigResponse {
    pub memory_address: Addr,
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
    pub os_id: u32,
    pub account: String,
    pub channel_id: String,
}
