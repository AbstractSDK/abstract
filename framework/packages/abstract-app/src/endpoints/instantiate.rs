use abstract_std::{
    app::{AppState, BaseInstantiateMsg, InstantiateMsg},
    objects::module_version::set_module_data,
};
use cosmwasm_std::{DepsMut, Env, MessageInfo, Response};
use cw2::set_contract_version;
use schemars::JsonSchema;
use serde::{de::DeserializeOwned, Serialize};

use crate::{
    state::{AppContract, ContractError},
    Handler, InstantiateEndpoint,
};

impl<
        Error: ContractError,
        CustomInitMsg: Serialize + DeserializeOwned + JsonSchema,
        CustomExecMsg,
        CustomQueryMsg,
        CustomMigrateMsg,
        SudoMsg,
    > InstantiateEndpoint
    for AppContract<Error, CustomInitMsg, CustomExecMsg, CustomQueryMsg, CustomMigrateMsg, SudoMsg>
{
    type InstantiateMsg = InstantiateMsg<Self::CustomInitMsg>;
    fn instantiate(
        self,
        mut deps: DepsMut,
        env: Env,
        info: MessageInfo,
        msg: Self::InstantiateMsg,
    ) -> Result<Response, Error> {
        let BaseInstantiateMsg { account } = msg.base;

        let module_msg = msg.module;

        // Base state
        let state = AppState {
            account: account.clone(),
        };
        let (name, version, metadata) = self.info();
        set_module_data(deps.storage, name, version, self.dependencies(), metadata)?;
        set_contract_version(deps.storage, name, version)?;
        self.base_state.save(deps.storage, &state)?;
        self.admin.set(deps.branch(), Some(account.into_addr()))?;

        let Some(handler) = self.maybe_instantiate_handler() else {
            return Ok(Response::new());
        };
        handler(deps, env, info, self, module_msg)
    }
}

#[cfg(test)]
mod test {
    #![allow(clippy::needless_borrows_for_generic_args)]
    use super::InstantiateMsg as SuperInstantiateMsg;
    use crate::mock::*;
    use abstract_sdk::base::InstantiateEndpoint;
    use abstract_std::app::BaseInstantiateMsg;
    use abstract_testing::prelude::*;

    #[coverage_helper::test]
    fn test_instantiate() {
        let mut deps = mock_dependencies();
        let abstr = AbstractMockAddrs::new(deps.api);

        let info = message_info(&abstr.module_factory, &[]);
        let env = mock_env_validated(deps.api);

        deps.querier = app_base_mock_querier(deps.api).build();

        let msg = SuperInstantiateMsg {
            base: BaseInstantiateMsg {
                account: abstr.account,
            },
            module: MockInitMsg {},
        };

        let res = MOCK_APP_WITH_DEP
            .instantiate(deps.as_mut(), env, info, msg)
            .unwrap();
        assert!(res.messages.is_empty());
    }
}
