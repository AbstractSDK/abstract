use crate::IcaAction;
use abstract_sdk::std::objects::TruncatedChainId;
use cosmwasm_schema::QueryResponses;
use cosmwasm_std::{Addr, CosmosMsg};

pub mod state {

    use abstract_sdk::feature_objects::{AnsHost, VersionControlContract};
    use cw_storage_plus::Item;

    #[cosmwasm_schema::cw_serde]
    pub struct Config {
        pub version_control: VersionControlContract,
        pub ans_host: AnsHost,
    }

    pub const CONFIG: Item<Config> = Item::new("config");
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
        actions: Vec<IcaAction>,
    },
}

#[cosmwasm_schema::cw_serde]
pub struct ConfigResponse {
    pub ans_host: String,
    pub version_control_address: String,
}

#[cosmwasm_schema::cw_serde]
pub struct IcaActionResult {
    /// messages that call the underlying implementations (be it polytone/cw-ica-controller/etc)
    pub msgs: Vec<CosmosMsg>,
}
