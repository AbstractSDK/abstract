use crate::{
    ibc::CallbackInfo,
    ibc_host::HostAction,
    objects::{account::AccountId, chain_name::ChainName},
};
use cosmwasm_schema::QueryResponses;
use cosmwasm_std::Coin;
use polytone::callbacks::CallbackMessage;

pub use polytone::callbacks::CallbackRequest;

pub mod state {

    use crate::objects::{
        account::AccountId, ans_host::AnsHost, chain_name::ChainName,
        common_namespace::ADMIN_NAMESPACE,
    };
    use cosmwasm_std::Addr;
    use cw_controllers::Admin;
    use cw_storage_plus::{Item, Map};

    #[cosmwasm_schema::cw_serde]
    pub struct Config {
        pub version_control: Addr,
        pub ans_host: AnsHost,
    }

    pub const ADMIN: Admin = Admin::new(ADMIN_NAMESPACE);

    // Saves the local note deployed contract
    // This allows sending cross-chain messages
    pub const POLYTONE_NOTE: Map<&ChainName, Addr> = Map::new("polytone_note");
    pub const REVERSE_POLYTONE_NOTE: Map<&Addr, ChainName> = Map::new("reverse-polytone_note");
    // Saves the remote ibc host addresses
    pub const REMOTE_HOST: Map<&ChainName, String> = Map::new("abstract-ibc-hosts");
    // Saves the remote polytone proxy addreses corresponding to that contract
    // This is used for executing message on the host
    pub const REMOTE_PROXY: Map<&ChainName, String> = Map::new("abstract-ibc-client-proxy");

    pub const CONFIG: Item<Config> = Item::new("config");
    /// (account_id, chain_name) -> remote proxy account address
    pub const ACCOUNTS: Map<(&AccountId, &ChainName), String> = Map::new("accounts");

    // For callbacks tests
    pub const ACKS: Item<Vec<String>> = Item::new("temp-callback-storage");
}

/// This needs no info. Owner of the contract is whoever signed the InstantiateMsg.
#[cosmwasm_schema::cw_serde]
pub struct InstantiateMsg {
    pub ans_host_address: String,
    pub version_control_address: String,
}

#[cosmwasm_schema::cw_serde]
pub struct MigrateMsg {}

#[cosmwasm_schema::cw_serde]
#[cfg_attr(feature = "interface", derive(cw_orch::ExecuteFns))]
pub enum ExecuteMsg {
    /// Update the Admin
    UpdateAdmin {
        admin: String,
    },
    // Registers the polytone note on the local chain as well as the host on the remote chain to send messages through
    // This allows for monitoring which chain are connected to the contract remotely
    RegisterChainHost {
        chain: String,
        note: String,
        host: String,
    },
    /// Changes the config
    UpdateConfig {
        ans_host: Option<String>,
        version_control: Option<String>,
    },
    /// Only callable by Account proxy
    /// Will attempt to forward the specified funds to the corresponding
    /// address on the remote chain.
    SendFunds {
        host_chain: String,
        funds: Vec<Coin>,
    },
    /// Register an Account on a remote chain over IBC
    /// This action creates a proxy for them on the remote chain.
    Register {
        host_chain: String,
    },
    RemoteAction {
        // host chain to be executed on
        // Example: "osmosis"
        host_chain: String,
        // execute the custom host function
        action: HostAction,
        // optional callback info
        callback_info: Option<CallbackInfo>,
    },
    RemoveHost {
        host_chain: String,
    },

    /// Callback from the Polytone implementation
    /// This is only triggered when a contract execution is succesful
    Callback(CallbackMessage),
}

/// This enum is used for sending callbacks to the note contract of the IBC client
#[cosmwasm_schema::cw_serde]
pub enum IbcClientCallback {
    ExecuteAction {
        receiver: String,
        callback_id: String,
    },
    CreateAccount {
        account_id: AccountId,
    },
    WhoAmI {},
}

#[cosmwasm_schema::cw_serde]
#[cfg_attr(feature = "interface", derive(cw_orch::QueryFns))]
#[derive(QueryResponses)]
pub enum QueryMsg {
    // Returns config
    #[returns(HostResponse)]
    Config {},
    // Returns config
    #[returns(HostResponse)]
    Host { chain_name: String },
    // Shows all open channels (incl. remote info)
    #[returns(ListAccountsResponse)]
    ListAccounts {
        start: Option<(AccountId, String)>,
        limit: Option<u32>,
    },
    // Get channel info for one chain
    #[returns(AccountResponse)]
    Account {
        chain: String,
        account_id: AccountId,
    },
    // get the hosts
    #[returns(ListRemoteHostsResponse)]
    ListRemoteHosts {},
    // get the IBC execution proxys
    #[returns(ListRemoteProxysResponse)]
    ListRemoteProxys {},
}

#[cosmwasm_schema::cw_serde]
pub struct ConfigResponse {
    pub admin: String,
    pub ans_host: String,
    pub version_control_address: String,
}

#[cosmwasm_schema::cw_serde]
pub struct ListAccountsResponse {
    pub accounts: Vec<(AccountId, ChainName, String)>,
}
#[cosmwasm_schema::cw_serde]
pub struct ListRemoteHostsResponse {
    pub hosts: Vec<(ChainName, String)>,
}
#[cosmwasm_schema::cw_serde]
pub struct ListRemoteProxysResponse {
    pub proxys: Vec<(ChainName, String)>,
}

#[cosmwasm_schema::cw_serde]
pub struct HostResponse {
    pub remote_host: Option<String>,
    pub remote_polytone_proxy: Option<String>,
}
#[cosmwasm_schema::cw_serde]
pub struct AccountResponse {
    pub remote_proxy_addr: String,
}

#[cosmwasm_schema::cw_serde]
pub struct RemoteProxyResponse {
    /// last block balance was updated (0 is never)
    pub channel_id: String,
    /// address of the remote proxy
    pub proxy_address: String,
}
