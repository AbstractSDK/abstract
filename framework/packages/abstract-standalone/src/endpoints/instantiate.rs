use abstract_sdk::AbstractSdkResult;
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
        _msg: StandaloneInstantiateMsg,
        is_migratable: bool,
    ) -> AbstractSdkResult<()> {
        let account =
            abstract_std::module_factory::state::CURRENT_BASE.query(&deps.querier, info.sender)?;

        // Base state
        let state = StandaloneState {
            account: account.clone(),
            is_migratable,
        };
        let (name, version, metadata) = self.info;
        set_module_data(deps.storage, name, version, self.dependencies, metadata)?;
        set_contract_version(deps.storage, name, version)?;
        self.base_state.save(deps.storage, &state)?;
        self.admin.set(deps, Some(account.into_addr()))?;
        Ok(())
    }
}

#[cfg(test)]
mod test {
    use crate::mock::*;
    use abstract_std::standalone::StandaloneInstantiateMsg;
    use abstract_unit_test_utils::prelude::*;
    use cosmwasm_std::testing::message_info;

    #[coverage_helper::test]
    fn test_instantiate() {
        let mut deps = mock_dependencies();
        let abstr = AbstractMockAddrs::new(deps.api);
        deps.querier = standalone_base_mock_querier(deps.api).build();

        let env = mock_env_validated(deps.api);
        let info = message_info(&abstr.module_factory, &[]);
        let msg = MockInitMsg {
            base: StandaloneInstantiateMsg {},
            migratable: true,
        };

        let res = instantiate(deps.as_mut(), env, info, msg).unwrap();
        assert!(res.messages.is_empty());
    }
}
