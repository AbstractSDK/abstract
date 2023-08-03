use crate::{
    endpoints::reply::{
        reply_dispatch_callback, reply_init_callback, INIT_CALLBACK_ID, RECEIVE_DISPATCH_ID,
    },
    HostError,
};
use abstract_core::{
    objects::{chain_name::ChainName, AccountId},
    version_control::AccountBase,
};
use abstract_sdk::{
    base::{
        AbstractContract, ExecuteHandlerFn, InstantiateHandlerFn, QueryHandlerFn, ReceiveHandlerFn,
        ReplyHandlerFn, SudoHandlerFn,
    },
    core::ibc_host::PacketMsg,
    feature_objects::AnsHost,
    namespaces::{ADMIN_NAMESPACE, BASE_STATE},
    AbstractSdkError,
};
use cosmwasm_schema::cw_serde;
use cosmwasm_std::{Addr, Binary, Empty, StdResult, Storage};
use cw_controllers::Admin;
use cw_storage_macro::index_list;
use cw_storage_plus::{IndexList, IndexedMap, Item, Map, MultiIndex, UniqueIndex};
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

/// Store channel information for account creation reply
pub const REGISTRATION_CACHE: Item<(String, AccountId)> = Item::new("rc");
/// Store the processing packet information for processing in Reply along with the channel id it came from
pub const PROCESSING_PACKET: Item<(PacketMsg, String)> = Item::new("pr");
/// account_id -> client_proxy_addr
pub const CLIENT_PROXY: Map<&AccountId, String> = Map::new("cp");
/// Maps a channel to its chain name
pub const CHAIN_OF_CHANNEL: Map<&str, ChainName> = Map::new("cac");
/// Maps a chain name to its client proxy address
pub const CHAIN_CLIENTS: Map<&ChainName, String> = Map::new("ccl");
// this stores all results from current dispatch
pub const RESULTS: Item<Vec<Binary>> = Item::new("res");
/// Configuration of the IBC host
pub const CONFIG: Item<'static, Config> = Item::new("cfg");
/// The BaseState contains the main addresses needed for sending and verifying messages
#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct Config {
    /// AnsHost contract struct (address)
    pub ans_host: AnsHost,
    /// Address of the account factory, used to create remote accounts
    pub account_factory: Addr,
    /// Address of the local version control, for retrieving account information
    pub version_control: Addr,
}

// #[cw_serde]
// pub struct AbstractChannel {
//     client_chain: ChainName,
//     channel_id: String,
//     client_address: Addr,
// }
// #[index_list(AbstractChannel)]
// struct ChannelIndexes<'a> {
//     client_chain: MultiIndex<'a, &'a ChainName, AbstractChannel, String>,
//     channel_id: UniqueIndex<'a, String, AbstractChannel>,
// }

// pub fn tokens<'a>() -> IndexedMap<'a, &'a str, TokenInfo, ChannelIndexes<'a>> {
//     let indexes = ChannelIndexes {
//         client_chain: MultiIndex::new(
//             |d: , AbstractChannel| d.clone(),
//             "tokens",
//             "tokens__owner",
//         ),
//     };
//     IndexedMap::new("tokens", indexes)
// }
