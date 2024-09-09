use cosmwasm_std::Binary;

use crate::objects::module::ModuleInfo;

/// Module info and init message
#[non_exhaustive]
#[cosmwasm_schema::cw_serde]
pub struct ModuleInstallConfig {
    pub module: ModuleInfo,
    pub init_msg: Option<Binary>,
}

impl ModuleInstallConfig {
    pub fn new(module: ModuleInfo, init_msg: Option<Binary>) -> Self {
        Self { module, init_msg }
    }
}

/// Internal configuration actions accessible from the [`ExecuteMsg::UpdateInternalConfig`] message.
#[cosmwasm_schema::cw_serde]
#[non_exhaustive]
pub enum InternalConfigAction {
    /// Updates the [`state::ACCOUNT_MODULES`] map
    /// Only callable by account factory or owner.
    UpdateModuleAddresses {
        to_add: Option<Vec<(String, String)>>,
        to_remove: Option<Vec<String>>,
    },
}

#[cosmwasm_schema::cw_serde]
#[non_exhaustive]
pub enum UpdateSubAccountAction {
    /// Unregister sub-account
    /// It will unregister sub-account from the state
    /// Could be called only by the sub-account itself
    UnregisterSubAccount { id: u32 },
    /// Register sub-account
    /// It will register new sub-account into the state
    /// Could be called by the sub-account manager
    /// Note: since it happens after the claim by this manager state won't have spam accounts
    RegisterSubAccount { id: u32 },
}
