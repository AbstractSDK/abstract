use abstract_sdk::{
    feature_objects::{AnsHost, VersionControlContract},
    AbstractSdkResult,
};
use abstract_std::{
    objects::module_version::set_module_data,
    standalone::{StandaloneInstantiateMsg, StandaloneState},
};
use cosmwasm_std::{DepsMut, MessageInfo};
use cw2::set_contract_version;

use crate::state::StandaloneContract;

impl StandaloneContract {
    /// Instantiates the `Standalone` state for this contract.
    ///
    /// **Note:** This contract can only be instantiated by the abstract module factory.
    pub fn instantiate(
        &self,
        deps: DepsMut,
        info: MessageInfo,
        msg: StandaloneInstantiateMsg,
        is_migratable: bool,
    ) -> AbstractSdkResult<()> {
        let StandaloneInstantiateMsg {
            ans_host_address,
            version_control_address,
        } = msg;
        let ans_host = AnsHost {
            address: deps.api.addr_validate(&ans_host_address)?,
        };
        let version_control = VersionControlContract {
            address: deps.api.addr_validate(&version_control_address)?,
        };
        let account_base =
            abstract_std::module_factory::state::CURRENT_BASE.query(&deps.querier, info.sender)?;

        // Base state
        let state = StandaloneState {
            proxy_address: account_base.proxy,
            ans_host,
            version_control,
            is_migratable,
        };
        let (name, version, metadata) = self.info;
        set_module_data(deps.storage, name, version, self.dependencies, metadata)?;
        set_contract_version(deps.storage, name, version)?;
        self.base_state.save(deps.storage, &state)?;
        self.admin.set(deps, Some(account_base.manager))?;
        Ok(())
    }
}
