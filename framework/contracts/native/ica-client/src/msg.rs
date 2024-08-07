use abstract_ica::IcaAction;
use abstract_std::objects::TruncatedChainId;
use cosmwasm_schema::QueryResponses;
use cosmwasm_std::{Addr, CosmosMsg};

pub mod state {

    use abstract_sdk::feature_objects::{AnsHost, VersionControlContract};
    use abstract_std::objects::TruncatedChainId;
    use cosmwasm_std::Addr;
    use cw_storage_plus::{Item, Map};

    #[cosmwasm_schema::cw_serde]
    pub struct Config {
        pub version_control: VersionControlContract,
        pub ans_host: AnsHost,
    }

    /// Information about the deployed infrastructure we're connected to.
    #[cosmwasm_schema::cw_serde]
    pub struct IbcInfrastructure {
        /// Address of the polytone note deployed on the local chain. This contract will forward the messages for us.
        pub polytone_note: Addr,
        /// The address of the abstract host deployed on the remote chain. This address will be called with our packet.
        pub remote_abstract_host: String,
        // The remote polytone proxy address which will be called by the polytone host.
        pub remote_proxy: Option<String>,
    }

    // Saves the local note deployed contract and the remote abstract host connected
    // This allows sending cross-chain messages
    pub const IBC_INFRA: Map<&TruncatedChainId, IbcInfrastructure> = Map::new("ibci");
    pub const REVERSE_POLYTONE_NOTE: Map<&Addr, TruncatedChainId> = Map::new("revpn");

    pub const CONFIG: Item<Config> = Item::new("config");

    // For callbacks tests
    pub const ACKS: Item<Vec<String>> = Item::new("tmpc");
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
#[derive(cw_orch::ExecuteFns)]
pub enum ExecuteMsg {}

#[cosmwasm_schema::cw_serde]
#[derive(QueryResponses, cw_orch::QueryFns)]
pub enum QueryMsg {
    /// Queries the ownership of the ibc client contract
    /// Returns [`cw_ownable::Ownership<Addr>`]
    #[returns(cw_ownable::Ownership<Addr> )]
    Ownership {},

    /// Returns config
    /// Returns [`ConfigResponse`]
    #[returns(ConfigResponse)]
    Config {},

    #[returns(IcaActionResult)]
    IcaAction {
        // Proxy address used to query polytone implementations or proxy itself.
        proxy_address: String,
        // Chain to send to
        chain: TruncatedChainId,
        // Queries go first
        action: Vec<IcaAction>,
    },
}

#[cosmwasm_schema::cw_serde]
pub struct ConfigResponse {
    pub ans_host: String,
    pub version_control_address: String,
}

#[cosmwasm_schema::cw_serde]
struct IcaActionResult {
    /// messages that call the underlying implementations (be it polytone/cw-ica-controller/etc)
    msgs: Vec<CosmosMsg>,
}
