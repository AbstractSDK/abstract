use crate::{
    state::{AppContract, AppState, ContractError},
    Handler, InstantiateEndpoint,
};
use abstract_core::{
    app::{BaseInstantiateMsg, InstantiateMsg},
    objects::module_version::set_module_data,
};
use abstract_sdk::{feature_objects::{AnsHost, VersionControlContract}, features::{DepsAccess, ResponseGenerator}};
use cosmwasm_std::{DepsMut, Env, MessageInfo, Response};
use cw2::set_contract_version;
use schemars::JsonSchema;
use serde::{de::DeserializeOwned, Serialize};

impl<
        'a,
        Error: ContractError,
        CustomInitMsg: Serialize + DeserializeOwned + JsonSchema,
        CustomExecMsg,
        CustomQueryMsg,
        CustomMigrateMsg,
        ReceiveMsg,
        SudoMsg,
    > InstantiateEndpoint
    for AppContract<
        'a,
        Error,
        CustomInitMsg,
        CustomExecMsg,
        CustomQueryMsg,
        CustomMigrateMsg,
        ReceiveMsg,
        SudoMsg,
    >
{
    type InstantiateMsg = InstantiateMsg<Self::CustomInitMsg>;
    fn instantiate(mut self, msg: Self::InstantiateMsg) -> Result<Response, Error> {
        let BaseInstantiateMsg {
            ans_host_address,
            version_control_address,
            account_base,
        } = msg.base;

        let module_msg = msg.module;

        let ans_host = AnsHost {
            address: self.api().addr_validate(&ans_host_address)?,
        };
        let version_control = VersionControlContract {
            address: self.api().addr_validate(&version_control_address)?,
        };

        // Base state
        let state = AppState {
            proxy_address: account_base.proxy.clone(),
            ans_host,
            version_control,
        };
        let (name, version, metadata) = self.info();
        let dependencies = self.dependencies();
        set_module_data(
            self.deps_mut().storage,
            name.clone(),
            version.clone(),
            dependencies,
            metadata,
        )?;
        set_contract_version(self.deps_mut().storage, &name, &version)?;
        self.base_state.save(self.deps.deps_mut().storage, &state)?;
        self.admin
            .set(self.deps.deps_mut(), Some(account_base.manager))?;

        let Some(handler) = self.maybe_instantiate_handler() else {
            return Ok(Response::new());
        };
        handler(&mut self, module_msg)?;
        Ok(self._generate_response()?)
    }
}

#[cfg(test)]
mod test {
    use super::InstantiateMsg as SuperInstantiateMsg;
    use crate::mock::*;
    use abstract_core::app::BaseInstantiateMsg;
    use abstract_sdk::base::InstantiateEndpoint;
    use speculoos::prelude::*;

    use abstract_testing::{
        addresses::test_account_base,
        prelude::{TEST_ANS_HOST, TEST_MODULE_FACTORY, TEST_VERSION_CONTROL},
    };
    use speculoos::assert_that;

    #[test]
    fn test_instantiate() {
        let mut deps = mock_dependencies();
        let info = mock_info(TEST_MODULE_FACTORY, &[]);

        deps.querier = app_base_mock_querier().build();

        let msg = SuperInstantiateMsg {
            base: BaseInstantiateMsg {
                ans_host_address: TEST_ANS_HOST.to_string(),
                version_control_address: TEST_VERSION_CONTROL.to_string(),
                account_base: test_account_base(),
            },
            module: MockInitMsg {},
        };

        let res = mock_app((deps.as_mut(), mock_env(), info).into())
            .instantiate(msg)
            .unwrap();
        assert_that!(res.messages).is_empty();
    }
}
