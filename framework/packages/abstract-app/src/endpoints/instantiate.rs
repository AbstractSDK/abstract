use crate::{
    state::{AppContract, AppState, ContractError},
    Handler, InstantiateEndpoint,
};
use abstract_core::{
    app::{BaseInstantiateMsg, InstantiateMsg},
    objects::module_version::set_module_data,
};
use abstract_sdk::{
    core::module_factory::{ContextResponse, QueryMsg as FactoryQuery},
    cw_helpers::wasm_smart_query,
    feature_objects::{AnsHost, VersionControlContract},
    features::{DepsAccess, DepsMutAccess, ResponseGenerator},
};
use cosmwasm_std::{DepsMut, Env, MessageInfo, Response};
use cw2::set_contract_version;
use schemars::JsonSchema;
use serde::Serialize;

impl<
        'a,
        Error: ContractError,
        CustomInitMsg: Serialize + JsonSchema,
        CustomExecMsg,
        CustomQueryMsg,
        CustomMigrateMsg,
        ReceiveMsg,
        SudoMsg,
    > InstantiateEndpoint
    for AppContract<
        'a,
        (DepsMut<'a>, Env, MessageInfo),
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
        } = msg.base;
        let ans_host = AnsHost {
            address: self.api().addr_validate(&ans_host_address)?,
        };
        let version_control = VersionControlContract {
            address: self.api().addr_validate(&version_control_address)?,
        };

        // TODO: Would be nice to remove context
        // Issue: We can't pass easily AccountBase with BaseInstantiateMsg(right now)

        // Caller is factory so get proxy and manager (admin) from there
        let resp: ContextResponse = self.deps().querier.query(&wasm_smart_query(
            self.message_info().sender.to_string(),
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
        handler(&mut self, msg.module)?;

        Ok(self._generate_response()?)
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

        let ctx = (deps.as_mut(), mock_env(), info).into();
        let res = MOCK_APP.instantiate(ctx, msg).unwrap();
        assert_that!(res.messages).is_empty();
    }
}
