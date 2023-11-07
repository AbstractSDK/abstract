use crate::{
    state::{AppContract, AppState, ContractError},
    Handler, InstantiateEndpoint,
};
use abstract_core::{
    app::{BaseInstantiateMsg, InstantiateMsg},
    objects::module_version::set_module_data,
};
use abstract_sdk::{
    cw_helpers::wasm_smart_query,
    feature_objects::{AnsHost, VersionControlContract},
    framework::module_factory::{ContextResponse, QueryMsg as FactoryQuery},
};
use cosmwasm_std::{DepsMut, Env, MessageInfo, Response};
use cw2::set_contract_version;
use schemars::JsonSchema;
use serde::Serialize;

impl<
        Error: ContractError,
        CustomInitMsg: Serialize + JsonSchema,
        CustomExecMsg,
        CustomQueryMsg,
        CustomMigrateMsg,
        ReceiveMsg,
        SudoMsg,
    > InstantiateEndpoint
    for AppContract<
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
    fn instantiate(
        self,
        mut deps: DepsMut,
        env: Env,
        info: MessageInfo,
        msg: Self::InstantiateMsg,
    ) -> Result<Response, Error> {
        let BaseInstantiateMsg {
            ans_host_address,
            version_control_address,
        } = msg.base;
        let ans_host = AnsHost {
            address: deps.api.addr_validate(&ans_host_address)?,
        };
        let version_control = VersionControlContract {
            address: deps.api.addr_validate(&version_control_address)?,
        };

        // Caller is factory so get proxy and manager (admin) from there
        let resp: ContextResponse = deps.querier.query(&wasm_smart_query(
            info.sender.to_string(),
            &FactoryQuery::Context {},
        )?)?;

        let account_base = resp.account_base;

        // Base state
        let state = AppState {
            proxy_address: account_base.proxy.clone(),
            ans_host,
            version_control,
        };
        let (name, version, metadata) = self.info();
        set_module_data(deps.storage, name, version, self.dependencies(), metadata)?;
        set_contract_version(deps.storage, name, version)?;
        self.base_state.save(deps.storage, &state)?;
        self.admin.set(deps.branch(), Some(account_base.manager))?;

        let Some(handler) = self.maybe_instantiate_handler() else {
            return Ok(Response::new());
        };
        handler(deps, env, info, self, msg.module)
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use crate::mock::*;
    use speculoos::prelude::*;

    use abstract_testing::prelude::{TEST_ANS_HOST, TEST_MODULE_FACTORY, TEST_VERSION_CONTROL};
    use speculoos::assert_that;

    #[test]
    fn test_instantiate() {
        let mut deps = mock_dependencies();
        let info = mock_info(TEST_MODULE_FACTORY, &[]);

        deps.querier = app_base_mock_querier().build();

        let msg = InstantiateMsg {
            base: BaseInstantiateMsg {
                ans_host_address: TEST_ANS_HOST.to_string(),
                version_control_address: TEST_VERSION_CONTROL.to_string(),
            },
            module: MockInitMsg {},
        };

        let res = MOCK_APP
            .instantiate(deps.as_mut(), mock_env(), info, msg)
            .unwrap();
        assert_that!(res.messages).is_empty();
    }
}
