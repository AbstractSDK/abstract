use abstract_sdk::feature_objects::{AnsHost, VersionControlContract};
use abstract_std::{
    app::{BaseInstantiateMsg, InstantiateMsg},
    objects::module_version::set_module_data,
};
use cosmwasm_std::{DepsMut, Env, MessageInfo, Response};
use cw2::set_contract_version;
use schemars::JsonSchema;
use serde::{de::DeserializeOwned, Serialize};

use crate::{
    state::{AppContract, AppState, ContractError},
    Handler, InstantiateEndpoint,
};

impl<
        Error: ContractError,
        CustomInitMsg: Serialize + DeserializeOwned + JsonSchema,
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
            account_base,
        } = msg.base;

        let module_msg = msg.module;

        let ans_host = AnsHost {
            address: deps.api.addr_validate(&ans_host_address)?,
        };
        let version_control = VersionControlContract {
            address: deps.api.addr_validate(&version_control_address)?,
        };

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
        handler(deps, env, info, self, module_msg)
    }
}

#[cfg(test)]
mod test {
    use abstract_sdk::base::InstantiateEndpoint;
    use abstract_std::app::BaseInstantiateMsg;
    use abstract_testing::{
        addresses::test_account_base,
        prelude::{TEST_ANS_HOST, TEST_MODULE_FACTORY, TEST_VERSION_CONTROL},
    };
    use cosmwasm_std::testing::{mock_dependencies, mock_env, mock_info};
    use speculoos::{assert_that, prelude::*};

    use super::InstantiateMsg as SuperInstantiateMsg;
    use crate::mock::*;

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

        let res = MOCK_APP_WITH_DEP
            .instantiate(deps.as_mut(), mock_env(), info, msg)
            .unwrap();
        assert_that!(res.messages).is_empty();
    }
}
