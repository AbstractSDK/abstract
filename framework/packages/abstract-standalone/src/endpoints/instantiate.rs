use abstract_sdk::feature_objects::{AnsHost, VersionControlContract};
use abstract_std::{
    objects::module_version::set_module_data,
    standalone::{BaseInstantiateMsg, StandaloneState},
};
use cosmwasm_std::{DepsMut, StdResult};
use cw2::set_contract_version;

use crate::state::StandaloneContract;

impl StandaloneContract {
    /// Call this method on instantiating of the standalone
    pub fn instantiate(&self, deps: DepsMut, msg: BaseInstantiateMsg) -> StdResult<()> {
        let BaseInstantiateMsg {
            ans_host_address,
            version_control_address,
        } = msg;

        let ans_host = AnsHost {
            address: deps.api.addr_validate(&ans_host_address)?,
        };
        let version_control = VersionControlContract {
            address: deps.api.addr_validate(&version_control_address)?,
        };

        // Base state
        let state = StandaloneState {
            ans_host,
            version_control,
        };
        let (name, version, metadata) = self.info;
        set_module_data(deps.storage, name, version, &[], metadata)?;
        set_contract_version(deps.storage, name, version)?;
        self.base_state.save(deps.storage, &state)?;
        Ok(())
    }
}

#[cfg(test)]
mod test {
    use crate::mock::*;
    use abstract_std::standalone;
    use abstract_testing::prelude::{TEST_ANS_HOST, TEST_VERSION_CONTROL};

    #[test]
    fn test_instantiate() {
        let mut deps = mock_dependencies();

        deps.querier = standalone_base_mock_querier().build();

        let msg = MockInitMsg {
            base: standalone::BaseInstantiateMsg {
                ans_host_address: TEST_ANS_HOST.to_string(),
                version_control_address: TEST_VERSION_CONTROL.to_string(),
            },
        };

        BASIC_MOCK_STANDALONE
            .instantiate(deps.as_mut(), msg.base)
            .unwrap();
    }
}
