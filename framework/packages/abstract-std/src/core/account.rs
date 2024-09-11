use cosmwasm_schema::QueryResponses;
use cosmwasm_std::{Addr, Binary, CosmosMsg, Empty};
pub use manager::{
    InfoResponse, ModuleAddressesResponse, ModuleInfosResponse, ModuleInstallConfig,
    ModuleVersionsResponse, SubAccountIdsResponse, UpdateSubAccountAction,
};
use state::SuspensionStatus;

use crate::objects::{
    gov_type::{GovAction, GovernanceDetails, TopLevelOwnerResponse},
    module::ModuleInfo,
    ownership::Ownership,
    AccountId,
};

use super::*;

// TODO: Move manager and proxy state here
pub mod state {
    use super::*;

    pub use manager::state::{SuspensionStatus, ACCOUNT_ID, ACCOUNT_MODULES};
    pub use proxy::state::{State, ADMIN, STATE};
}

#[cosmwasm_schema::cw_serde]
pub struct MigrateMsg {}

/// Account Instantiate Msg
/// https://github.com/burnt-labs/contracts/blob/main/contracts/account/src/msg.rs
#[cosmwasm_schema::cw_serde]
pub struct InstantiateMsg {
    // TODO: fork and make pub
    // pub authenticator: Option<AddAuthenticator>,
    pub account_id: Option<AccountId>,
    pub owner: GovernanceDetails<String>,
    pub namespace: Option<String>,
    // Optionally modules can be provided. They will be installed after account registration.
    pub install_modules: Vec<ModuleInstallConfig>,
    pub name: String,
    pub description: Option<String>,
    pub link: Option<String>,
    // TODO: Compute these using instantiate2.
    pub module_factory_address: String,
    pub version_control_address: String,
}

#[cosmwasm_schema::cw_serde]
#[derive(cw_orch::ExecuteFns)]
pub enum ExecuteMsg {
    // ## Old Proxy ##
    /// Executes the provided messages if sender is whitelisted
    ModuleAction { msgs: Vec<CosmosMsg<Empty>> },
    /// Execute a message and forward the Response data
    ModuleActionWithData { msg: CosmosMsg<Empty> },
    /// Execute IBC action on Client
    IbcAction { msg: crate::ibc_client::ExecuteMsg },
    /// Queries the Abstract Ica Client with the provided action query.
    /// Provides access to different ICA implementations for different ecosystems.
    IcaAction {
        /// Query of type `abstract-ica-client::msg::QueryMsg`
        action_query_msg: Binary,
    },

    // ## Old Manager ##
    /// Forward execution message to module
    #[cw_orch(payable)]
    ExecOnModule { module_id: String, exec_msg: Binary },
    /// Update Abstract-specific configuration of the module.
    /// Only callable by the account factory or owner.
    UpdateInternalConfig(Binary),
    /// Install module using module factory, callable by Owner
    #[cw_orch(payable)]
    InstallModules {
        // Module information and Instantiate message to instantiate the contract
        modules: Vec<ModuleInstallConfig>,
    },
    /// Uninstall a module given its ID.
    UninstallModule { module_id: String },
    /// Upgrade the module to a new version
    /// If module is `abstract::manager` then the contract will do a self-migration.
    Upgrade {
        modules: Vec<(ModuleInfo, Option<Binary>)>,
    },
    /// Creates a sub-account on the account
    #[cw_orch(payable)]
    CreateSubAccount {
        // Name of the sub-account
        name: String,
        // Description of the account
        description: Option<String>,
        // URL linked to the account
        link: Option<String>,
        // optionally specify a namespace for the sub-account
        namespace: Option<String>,
        // Provide list of module to install after sub-account creation
        install_modules: Vec<ModuleInstallConfig>,
        /// If `None`, will create a new local account without asserting account-id.
        ///
        /// When provided sequence in 0..2147483648 range: The tx will error
        /// When provided sequence in 2147483648..u32::MAX range: Signals use of unclaimed Account Id in this range. The tx will error if this account-id already claimed. Useful for instantiate2 address prediction.
        account_id: Option<u32>,
    },
    /// Update info
    UpdateInfo {
        name: Option<String>,
        description: Option<String>,
        link: Option<String>,
    },
    /// Update account statuses
    UpdateStatus { is_suspended: Option<bool> },
    /// Actions called by internal or external sub-accounts
    UpdateSubAccount(UpdateSubAccountAction),
    /// Update the contract's ownership. The `action`
    /// can propose transferring ownership to an account,
    /// accept a pending ownership transfer, or renounce the ownership
    /// of the account permanently.
    UpdateOwnership(GovAction),
}

#[cosmwasm_schema::cw_serde]
#[derive(QueryResponses, cw_orch::QueryFns)]
pub enum QueryMsg {
    // ## Old Proxy ##
    /// Contains the enabled modules
    /// Returns [`ConfigResponse`]
    #[returns(ConfigResponse)]
    Config {},

    // ## Old Manager ##
    /// Query the versions of modules installed on the account given their `ids`.
    /// Returns [`ModuleVersionsResponse`]
    #[returns(ModuleVersionsResponse)]
    ModuleVersions { ids: Vec<String> },
    /// Query the addresses of modules installed on the account given their `ids`.
    /// Returns [`ModuleAddressesResponse`]
    #[returns(ModuleAddressesResponse)]
    ModuleAddresses { ids: Vec<String> },
    /// Query information of all modules installed on the account.
    /// Returns [`ModuleInfosResponse`]
    #[returns(ModuleInfosResponse)]
    ModuleInfos {
        start_after: Option<String>,
        limit: Option<u8>,
    },
    /// Query the Account info.
    /// Returns [`InfoResponse`]
    #[returns(InfoResponse)]
    Info {},
    /// Returns [`SubAccountIdsResponse`]
    #[returns(SubAccountIdsResponse)]
    SubAccountIds {
        start_after: Option<u32>,
        limit: Option<u8>,
    },
    /// Returns [`TopLevelOwnerResponse`]
    #[returns(TopLevelOwnerResponse)]
    TopLevelOwner {},
    /// Query the contract's ownership information
    #[returns(Ownership<String>)]
    Ownership {},
}

#[cosmwasm_schema::cw_serde]
pub struct ConfigResponse {
    pub modules: Vec<String>,
    pub account_id: AccountId,
    pub is_suspended: SuspensionStatus,
    pub version_control_address: Addr,
    pub module_factory_address: Addr,
}
