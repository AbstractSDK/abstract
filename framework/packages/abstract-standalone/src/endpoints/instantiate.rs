use abstract_sdk::{
    feature_objects::{AnsHost, VersionControlContract},
    AbstractSdkResult,
};
use abstract_std::{
    objects::module_version::set_module_data,
    standalone::{StandaloneInstantiateMsg, StandaloneState},
    AbstractError,
};
use cosmwasm_std::{Addr, DepsMut, Env};
use cw2::set_contract_version;

use crate::state::StandaloneContract;

impl StandaloneContract {
    /// Call this method on instantiating of the standalone
    pub fn instantiate(
        &self,
        deps: DepsMut,
        env: &Env,
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

        let contract_info = deps
            .querier
            .query_wasm_contract_info(&env.contract.address)?;
        let account_base = version_control
            .assert_manager(
                &Addr::unchecked(contract_info.admin.expect("module-factory set this")),
                &deps.querier,
            )
            .map_err(AbstractError::from)?;

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
