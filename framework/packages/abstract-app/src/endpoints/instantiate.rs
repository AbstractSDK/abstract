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
    feature_objects::AnsHost,
};
use cosmwasm_std::{DepsMut, Env, MessageInfo, Response, StdError};
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
        let BaseInstantiateMsg { ans_host_address } = msg.base;
        let ans_host = AnsHost {
            address: deps.api.addr_validate(&ans_host_address)?,
        };

        // Caller is factory so get proxy and manager (admin) from there
        let resp: ContextResponse = deps.querier.query(&wasm_smart_query(
            info.sender.to_string(),
            &FactoryQuery::Context {},
        )?)?;

        let Some(account_base) = resp.account_base else {
            return Err(
                StdError::generic_err("context of module factory not properly set.").into(),
            );
        };

        // Base state
        let state = AppState {
            proxy_address: account_base.proxy.clone(),
            ans_host,
        };
        let (name, version, metadata) = self.info();
        set_module_data(deps.storage, name, version, self.dependencies(), metadata)?;
        set_contract_version(deps.storage, name, version)?;
        self.base_state.save(deps.storage, &state)?;
        self.admin.set(deps.branch(), Some(account_base.manager))?;

        let Some(handler) = self.maybe_instantiate_handler() else {
            return Ok(Response::new())
        };
        handler(deps, env, info, self, msg.module)
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use crate::mock::*;
    use speculoos::prelude::*;

    use abstract_testing::prelude::{TEST_ANS_HOST, TEST_MODULE_FACTORY};
    use speculoos::assert_that;

    #[test]
    fn test_instantiate() {
        let mut deps = mock_dependencies();
        let info = mock_info(TEST_MODULE_FACTORY, &[]);

        deps.querier = app_base_mock_querier().build();

        let msg = InstantiateMsg {
            base: BaseInstantiateMsg {
                ans_host_address: TEST_ANS_HOST.to_string(),
            },
            module: MockInitMsg {},
        };

        let res = MOCK_APP
            .instantiate(deps.as_mut(), mock_env(), info, msg)
            .unwrap();
        assert_that!(res.messages).is_empty();
    }
}
