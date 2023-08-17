use abstract_core::objects::{chain_name::ChainName, AccountId};
use abstract_sdk::{core::ibc_host::PacketMsg, feature_objects::AnsHost};
use cosmwasm_std::{Addr, SubMsgResponse};

use cw_storage_plus::{Item, Map};
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

/// Store channel information for account creation reply
pub const REGISTRATION_CACHE: Item<AccountId> = Item::new("rc");
/// Store the processing packet information for processing in Reply along with the channel id it came from
pub const PROCESSING_PACKET: Item<(PacketMsg, String)> = Item::new("pr");
/// account_id -> client_proxy_addr
pub const CLIENT_PROXY: Map<&AccountId, String> = Map::new("cp");
/// Maps a chain name to the proxy it uses to interact on this local chain
pub const CHAIN_PROXYS: Map<&ChainName, Addr> = Map::new("ccl");
pub const REVERSE_CHAIN_PROXYS: Map<&Addr, ChainName> = Map::new("reverse-ccl");
// this stores all results from current dispatch
pub const RESULTS: Item<Vec<SubMsgResponse>> = Item::new("res");
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
