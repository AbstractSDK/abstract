use manager::ModuleInstallConfig;

use crate::objects::{gov_type::GovernanceDetails, AccountId};

use super::*;


/// Account Instantiate Msg
/// https://github.com/burnt-labs/contracts/blob/main/contracts/account/src/msg.rs
#[cosmwasm_schema::cw_serde]
pub struct InstantiateMsg {
    // TODO: fork and make pub
    // pub authenticator: AddAuthenticator,
    pub account_id: AccountId,
    // Optionally modules can be provided. They will be installed after account registration.
    pub install_modules: Vec<ModuleInstallConfig>,
    pub name: String,
    pub description: Option<String>,
    pub link: Option<String>,
    pub module_factory_address: String,
    pub version_control_address: String,
    pub ans_host_address: String,
}

#[cosmwasm_schema::cw_serde]
pub enum ExecMsg {
    Manager(manager::ExecuteMsg),
    Proxy(proxy::ExecuteMsg),
}

#[cosmwasm_schema::cw_serde]
pub enum QueryMsg {
    Manager(manager::QueryMsg),
    Proxy(proxy::QueryMsg),
}
