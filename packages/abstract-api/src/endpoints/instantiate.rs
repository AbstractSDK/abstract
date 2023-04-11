use crate::state::{ApiContract, ApiState, ContractError};
use abstract_core::{api::InstantiateMsg, objects::module_version::set_module_data};
use abstract_sdk::{
    base::{Handler, InstantiateEndpoint},
    feature_objects::AnsHost,
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
        ReceiveMsg,
        SudoMsg,
    > InstantiateEndpoint
    for ApiContract<Error, CustomInitMsg, CustomExecMsg, CustomQueryMsg, ReceiveMsg, SudoMsg>
{
    type InstantiateMsg = InstantiateMsg<CustomInitMsg>;
    /// Instantiate the api
    fn instantiate(
        self,
        deps: DepsMut,
        env: Env,
        info: MessageInfo,
        msg: Self::InstantiateMsg,
    ) -> Result<Response, Error> {
        let ans_host = AnsHost {
            address: deps.api.addr_validate(&msg.base.ans_host_address)?,
        };

        // Base state
        let state = ApiState {
            version_control: deps.api.addr_validate(&msg.base.version_control_address)?,
            ans_host,
        };
        let (name, version, metadata) = self.info();
        set_module_data(deps.storage, name, version, self.dependencies(), metadata)?;
        set_contract_version(deps.storage, name, version)?;
        self.base_state.save(deps.storage, &state)?;

        let Some(handler) = self.maybe_instantiate_handler() else {
            return Ok(Response::new())
        };
        handler(deps, env, info, self, msg.module)
    }
}

#[cfg(test)]
mod tests {
    use abstract_core::{
        api::{BaseInstantiateMsg, InstantiateMsg},
        objects::module_version::{ModuleData, MODULE},
    };
    use abstract_sdk::{base::InstantiateEndpoint, feature_objects::AnsHost};
    use cosmwasm_std::{
        testing::{mock_dependencies, mock_env, mock_info},
        Addr, StdError,
    };
    use cw2::{ContractVersion, CONTRACT};
    use speculoos::prelude::*;

    use crate::{
        mock::{ApiMockResult, MockInitMsg, MOCK_API, MOCK_DEP, TEST_METADATA},
        state::ApiState,
    };
    use abstract_testing::prelude::*;

    #[test]
    fn successful() -> ApiMockResult {
        let api = MOCK_API.with_dependencies(&[MOCK_DEP]);
        let env = mock_env();
        let info = mock_info(TEST_MANAGER, &[]);
        let mut deps = mock_dependencies();
        deps.querier = abstract_testing::mock_querier();
        let init_msg = InstantiateMsg {
            base: BaseInstantiateMsg {
                ans_host_address: TEST_ANS_HOST.into(),
                version_control_address: TEST_VERSION_CONTROL.into(),
            },
            module: MockInitMsg,
        };
        let res = api.instantiate(deps.as_mut(), env, info, init_msg)?;
        assert_that!(&res.messages.len()).is_equal_to(0);
        // confirm mock init handler executed
        assert_that!(&res.data).is_equal_to(Some("mock_init".as_bytes().into()));

        let module_data = MODULE.load(&deps.storage)?;
        assert_that!(module_data).is_equal_to(ModuleData {
            module: TEST_MODULE_ID.into(),
            version: TEST_VERSION.into(),
            dependencies: vec![(&crate::mock::MOCK_DEP).into()],
            metadata: Some(TEST_METADATA.into()),
        });

        let contract_version = CONTRACT.load(&deps.storage)?;
        assert_that!(contract_version).is_equal_to(ContractVersion {
            contract: TEST_MODULE_ID.into(),
            version: TEST_VERSION.into(),
        });

        let api = MOCK_API;
        let none_authorized = api.authorized_addresses.is_empty(&deps.storage);
        assert!(none_authorized);

        let state = api.base_state.load(&deps.storage)?;
        assert_that!(state).is_equal_to(ApiState {
            version_control: Addr::unchecked(TEST_VERSION_CONTROL),
            ans_host: AnsHost {
                address: Addr::unchecked(TEST_ANS_HOST),
            },
        });
        Ok(())
    }

    #[test]
    fn invalid_ans_host() -> ApiMockResult {
        let api = MOCK_API;
        let env = mock_env();
        let info = mock_info(TEST_MANAGER, &[]);
        let mut deps = mock_dependencies();
        deps.querier = abstract_testing::mock_querier();
        let init_msg = InstantiateMsg {
            base: BaseInstantiateMsg {
                ans_host_address: TEST_ANS_HOST.into(),
                version_control_address: "5".into(),
            },
            module: MockInitMsg,
        };
        let res = api.instantiate(deps.as_mut(), env, info, init_msg);
        assert_that!(&res).is_err_containing(
            &StdError::generic_err("Invalid input: human address too short for this mock implementation (must be >= 3).").into(),
        );
        Ok(())
    }

    #[test]
    fn invalid_version_control() -> ApiMockResult {
        let api = MOCK_API;
        let env = mock_env();
        let info = mock_info(TEST_MANAGER, &[]);
        let mut deps = mock_dependencies();
        deps.querier = abstract_testing::mock_querier();
        let init_msg = InstantiateMsg {
            base: BaseInstantiateMsg {
                ans_host_address: TEST_ANS_HOST.into(),
                version_control_address: "4".into(),
            },
            module: MockInitMsg,
        };
        let res = api.instantiate(deps.as_mut(), env, info, init_msg);
        assert_that!(&res).is_err_containing(
            &StdError::generic_err("Invalid input: human address too short for this mock implementation (must be >= 3).").into(),
        );
        Ok(())
    }
}
