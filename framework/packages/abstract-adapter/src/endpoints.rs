mod execute;
mod ibc_callback;
mod instantiate;
mod module_ibc;
mod query;
mod reply;
mod sudo;

#[macro_export]
macro_rules! export_endpoints {
    ($api_const:expr, $api_type:ty) => {
        $crate::__endpoints_without_custom__!($api_const, $api_type);
        /// Execute entrypoint
        #[::cosmwasm_std::entry_point]
        pub fn execute(
            deps: ::cosmwasm_std::DepsMut,
            env: ::cosmwasm_std::Env,
            info: ::cosmwasm_std::MessageInfo,
            msg: <$api_type as $crate::sdk::base::ExecuteEndpoint>::ExecuteMsg,
        ) -> Result<::cosmwasm_std::Response, <$api_type as $crate::sdk::base::Handler>::Error> {
            use $crate::sdk::base::ExecuteEndpoint;
            $api_const.execute(deps, env, info, msg)
        }
    };
    ($api_const:expr, $api_type:ty, $custom_exec:ty) => {
        $crate::__endpoints_without_custom__!($api_const, $api_type);
        /// Execute entrypoint
        #[::cosmwasm_std::entry_point]
        pub fn execute(
            deps: ::cosmwasm_std::DepsMut,
            env: ::cosmwasm_std::Env,
            info: ::cosmwasm_std::MessageInfo,
            msg: $custom_exec,
        ) -> Result<::cosmwasm_std::Response, <$api_type as $crate::sdk::base::Handler>::Error> {
            use $crate::sdk::base::{CustomExecuteHandler, ExecuteEndpoint};
            match CustomExecuteHandler::try_into_base(msg) {
                Ok(default) => $api_const.execute(deps, env, info, default),
                Err(custom) => custom.custom_execute(deps, env, info, $api_const),
            }
        }
    };
}

#[macro_export]
#[doc(hidden)]
macro_rules! __endpoints_without_custom__ {
    ($api_const:expr, $api_type:ty) => {
        /// Instantiate entrypoint
        #[::cosmwasm_std::entry_point]
        pub fn instantiate(
            deps: ::cosmwasm_std::DepsMut,
            env: ::cosmwasm_std::Env,
            info: ::cosmwasm_std::MessageInfo,
            msg: <$api_type as $crate::sdk::base::InstantiateEndpoint>::InstantiateMsg,
        ) -> Result<::cosmwasm_std::Response, <$api_type as $crate::sdk::base::Handler>::Error> {
            use $crate::sdk::base::InstantiateEndpoint;
            $api_const.instantiate(deps, env, info, msg)
        }

        /// Query entrypoint
        #[::cosmwasm_std::entry_point]
        pub fn query(
            deps: ::cosmwasm_std::Deps,
            env: ::cosmwasm_std::Env,
            msg: <$api_type as $crate::sdk::base::QueryEndpoint>::QueryMsg,
        ) -> Result<::cosmwasm_std::Binary, <$api_type as $crate::sdk::base::Handler>::Error> {
            use $crate::sdk::base::QueryEndpoint;
            $api_const.query(deps, env, msg)
        }

        // Reply entrypoint
        #[::cosmwasm_std::entry_point]
        pub fn reply(
            deps: ::cosmwasm_std::DepsMut,
            env: ::cosmwasm_std::Env,
            msg: ::cosmwasm_std::Reply,
        ) -> Result<::cosmwasm_std::Response, <$api_type as $crate::sdk::base::Handler>::Error> {
            use $crate::sdk::base::ReplyEndpoint;
            $api_const.reply(deps, env, msg)
        }

        // Sudo entrypoint
        #[::cosmwasm_std::entry_point]
        pub fn sudo(
            deps: ::cosmwasm_std::DepsMut,
            env: ::cosmwasm_std::Env,
            msg: <$api_type as $crate::sdk::base::Handler>::SudoMsg,
        ) -> Result<::cosmwasm_std::Response, <$api_type as $crate::sdk::base::Handler>::Error> {
            use $crate::sdk::base::SudoEndpoint;
            $api_const.sudo(deps, env, msg)
        }
    };
}

#[cfg(test)]
mod test {
    #![allow(clippy::needless_borrows_for_generic_args)]
    use abstract_sdk::base::{
        CustomExecuteHandler, ExecuteEndpoint, InstantiateEndpoint, QueryEndpoint, ReplyEndpoint,
        SudoEndpoint,
    };
    use abstract_std::adapter::{self, AdapterRequestMsg};
    use abstract_testing::prelude::*;
    use cosmwasm_std::{
        testing::{message_info, mock_dependencies, mock_env},
        Binary, SubMsgResult,
    };

    use crate::mock::*;

    #[test]
    fn exports_endpoints() {
        export_endpoints!(MOCK_ADAPTER, MockAdapterContract);

        let mut deps = mock_dependencies();
        let abstr = AbstractMockAddrs::new(deps.api);
        let owner = abstr.owner;

        // init
        let init_msg = adapter::InstantiateMsg {
            base: adapter::BaseInstantiateMsg {
                ans_host_address: abstr.ans_host.to_string(),
                version_control_address: abstr.version_control.to_string(),
            },
            module: MockInitMsg {},
        };
        let actual_init = instantiate(
            deps.as_mut(),
            mock_env_validated(deps.api),
            message_info(&owner, &[]),
            init_msg.clone(),
        );
        let expected_init = MOCK_ADAPTER.instantiate(
            deps.as_mut(),
            mock_env_validated(deps.api),
            message_info(&owner, &[]),
            init_msg,
        );
        assert_eq!(actual_init, expected_init);

        // exec
        let exec_msg = adapter::ExecuteMsg::Module(AdapterRequestMsg::new(None, MockExecMsg {}));
        let actual_exec = execute(
            deps.as_mut(),
            mock_env_validated(deps.api),
            message_info(&owner, &[]),
            exec_msg.clone(),
        );
        let expected_exec = MOCK_ADAPTER.execute(
            deps.as_mut(),
            mock_env_validated(deps.api),
            message_info(&owner, &[]),
            exec_msg,
        );
        assert_eq!(actual_exec, expected_exec);

        // query
        let query_msg = adapter::QueryMsg::Module(MockQueryMsg::GetSomething {});
        let actual_query = query(deps.as_ref(), mock_env_validated(deps.api), query_msg.clone());
        let expected_query = MOCK_ADAPTER.query(deps.as_ref(), mock_env_validated(deps.api), query_msg);
        assert_eq!(actual_query, expected_query);

        // sudo
        let sudo_msg = MockSudoMsg {};
        let actual_sudo = sudo(deps.as_mut(), mock_env_validated(deps.api), sudo_msg.clone());
        let expected_sudo = MOCK_ADAPTER.sudo(deps.as_mut(), mock_env_validated(deps.api), sudo_msg);
        assert_eq!(actual_sudo, expected_sudo);

        // reply
        let reply_msg = ::cosmwasm_std::Reply {
            id: 0,
            result: SubMsgResult::Err("test".into()),
            payload: Binary::default(),
            gas_used: 0,
        };
        let actual_reply = reply(deps.as_mut(), mock_env_validated(deps.api), reply_msg.clone());
        let expected_reply = MOCK_ADAPTER.reply(deps.as_mut(), mock_env_validated(deps.api), reply_msg);
        assert_eq!(actual_reply, expected_reply);
    }

    #[test]
    fn export_endpoints_custom() {
        #[cosmwasm_schema::cw_serde]
        #[derive(cw_orch::ExecuteFns)]
        pub enum CustomExecMsg {
            Base(abstract_std::adapter::BaseExecuteMsg),
            Module(AdapterRequestMsg<crate::mock::MockExecMsg>),
            IbcCallback(abstract_std::ibc::IbcResponseMsg),
            ModuleIbc(abstract_std::ibc::ModuleIbcMsg),
            Foo {},
        }

        impl From<crate::mock::MockExecMsg> for CustomExecMsg {
            fn from(request: crate::mock::MockExecMsg) -> Self {
                Self::Module(AdapterRequestMsg {
                    account_address: None,
                    request,
                })
            }
        }

        impl CustomExecuteHandler<MockAdapterContract> for CustomExecMsg {
            type ExecuteMsg = crate::mock::ExecuteMsg;

            fn try_into_base(self) -> Result<Self::ExecuteMsg, Self> {
                match self {
                    CustomExecMsg::Base(msg) => Ok(Self::ExecuteMsg::Base(msg)),
                    CustomExecMsg::Module(msg) => Ok(Self::ExecuteMsg::Module(msg)),
                    CustomExecMsg::IbcCallback(msg) => Ok(Self::ExecuteMsg::IbcCallback(msg)),
                    CustomExecMsg::ModuleIbc(msg) => Ok(Self::ExecuteMsg::ModuleIbc(msg)),
                    _ => Err(self),
                }
            }

            fn custom_execute(
                self,
                _deps: cosmwasm_std::DepsMut,
                _env: cosmwasm_std::Env,
                _info: cosmwasm_std::MessageInfo,
                _module: MockAdapterContract,
            ) -> Result<cosmwasm_std::Response, crate::mock::MockError> {
                Ok(cosmwasm_std::Response::new().set_data(b"foo"))
            }
        }
        export_endpoints!(MOCK_ADAPTER, MockAdapterContract, CustomExecMsg);

        let mut deps = mock_dependencies();

        let abstr = AbstractMockAddrs::new(deps.api);
        // custom
        let actual_custom_exec = execute(
            deps.as_mut(),
            mock_env_validated(deps.api),
            message_info(&abstr.owner, &[]),
            CustomExecMsg::Foo {},
        )
        .unwrap();
        let expected_custom_exec =
            cosmwasm_std::Response::<cosmwasm_std::Empty>::new().set_data(b"foo");
        assert_eq!(actual_custom_exec, expected_custom_exec);

        // ensure nothing got broken

        // init
        let init_msg = adapter::InstantiateMsg {
            base: adapter::BaseInstantiateMsg {
                ans_host_address: abstr.ans_host.to_string(),
                version_control_address: abstr.version_control.to_string(),
            },
            module: MockInitMsg {},
        };
        let actual_init = instantiate(
            deps.as_mut(),
            mock_env_validated(deps.api),
            message_info(&abstr.owner, &[]),
            init_msg.clone(),
        );
        let expected_init = MOCK_ADAPTER.instantiate(
            deps.as_mut(),
            mock_env_validated(deps.api),
            message_info(&abstr.owner, &[]),
            init_msg,
        );
        assert_eq!(actual_init, expected_init);

        // exec
        let exec_msg = MockExecMsg {};
        let actual_exec = execute(
            deps.as_mut(),
            mock_env_validated(deps.api),
            message_info(&abstr.owner, &[]),
            exec_msg.clone().into(),
        );
        let expected_exec = MOCK_ADAPTER.execute(
            deps.as_mut(),
            mock_env_validated(deps.api),
            message_info(&abstr.owner, &[]),
            exec_msg.into(),
        );
        assert_eq!(actual_exec, expected_exec);

        // query
        let query_msg = adapter::QueryMsg::Module(MockQueryMsg::GetSomething {});
        let actual_query = query(deps.as_ref(), mock_env_validated(deps.api), query_msg.clone());
        let expected_query = MOCK_ADAPTER.query(deps.as_ref(), mock_env_validated(deps.api), query_msg);
        assert_eq!(actual_query, expected_query);

        // sudo
        let sudo_msg = MockSudoMsg {};
        let actual_sudo = sudo(deps.as_mut(), mock_env_validated(deps.api), sudo_msg.clone());
        let expected_sudo = MOCK_ADAPTER.sudo(deps.as_mut(), mock_env_validated(deps.api), sudo_msg);
        assert_eq!(actual_sudo, expected_sudo);

        // reply
        let reply_msg = ::cosmwasm_std::Reply {
            id: 0,
            result: SubMsgResult::Err("test".into()),
            payload: Binary::default(),
            gas_used: 0,
        };
        let actual_reply = reply(deps.as_mut(), mock_env_validated(deps.api), reply_msg.clone());
        let expected_reply = MOCK_ADAPTER.reply(deps.as_mut(), mock_env_validated(deps.api), reply_msg);
        assert_eq!(actual_reply, expected_reply);
    }
}
