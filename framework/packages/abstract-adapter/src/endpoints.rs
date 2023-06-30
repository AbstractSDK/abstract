mod execute;
mod ibc_callback;
mod instantiate;
mod query;
mod receive;
mod reply;
mod sudo;

#[macro_export]
macro_rules! export_endpoints {
    ($api_const:expr, $api_type:ty) => {
        /// Instantiate entrypoint
        #[::cosmwasm_std::entry_point]
        pub fn instantiate(
            deps: ::cosmwasm_std::DepsMut,
            env: ::cosmwasm_std::Env,
            info: ::cosmwasm_std::MessageInfo,
            msg: <$api_type as ::abstract_sdk::base::InstantiateEndpoint>::InstantiateMsg,
        ) -> Result<::cosmwasm_std::Response, <$api_type as ::abstract_sdk::base::Handler>::Error> {
            use ::abstract_sdk::base::InstantiateEndpoint;
            $api_const.instantiate(deps, env, info, msg)
        }

        /// Execute entrypoint
        #[::cosmwasm_std::entry_point]
        pub fn execute(
            deps: ::cosmwasm_std::DepsMut,
            env: ::cosmwasm_std::Env,
            info: ::cosmwasm_std::MessageInfo,
            msg: <$api_type as ::abstract_sdk::base::ExecuteEndpoint>::ExecuteMsg,
        ) -> Result<::cosmwasm_std::Response, <$api_type as ::abstract_sdk::base::Handler>::Error> {
            use ::abstract_sdk::base::ExecuteEndpoint;
            $api_const.execute(deps, env, info, msg)
        }

        /// Query entrypoint
        #[::cosmwasm_std::entry_point]
        pub fn query(
            deps: ::cosmwasm_std::Deps,
            env: ::cosmwasm_std::Env,
            msg: <$api_type as ::abstract_sdk::base::QueryEndpoint>::QueryMsg,
        ) -> Result<::cosmwasm_std::Binary, <$api_type as ::abstract_sdk::base::Handler>::Error> {
            use ::abstract_sdk::base::QueryEndpoint;
            $api_const.query(deps, env, msg)
        }

        // Reply entrypoint
        #[::cosmwasm_std::entry_point]
        pub fn reply(
            deps: ::cosmwasm_std::DepsMut,
            env: ::cosmwasm_std::Env,
            msg: ::cosmwasm_std::Reply,
        ) -> Result<::cosmwasm_std::Response, <$api_type as ::abstract_sdk::base::Handler>::Error> {
            use ::abstract_sdk::base::ReplyEndpoint;
            $api_const.reply(deps, env, msg)
        }

        // Sudo entrypoint
        #[::cosmwasm_std::entry_point]
        pub fn sudo(
            deps: ::cosmwasm_std::DepsMut,
            env: ::cosmwasm_std::Env,
            msg: <$api_type as ::abstract_sdk::base::Handler>::SudoMsg,
        ) -> Result<::cosmwasm_std::Response, <$api_type as ::abstract_sdk::base::Handler>::Error> {
            use ::abstract_sdk::base::SudoEndpoint;
            $api_const.sudo(deps, env, msg)
        }
    };
}

#[cfg(test)]
mod test {
    use crate::mock::*;
    use abstract_core::adapter::{self, AdapterRequestMsg};
    use abstract_sdk::base::{
        ExecuteEndpoint, InstantiateEndpoint, QueryEndpoint, ReplyEndpoint, SudoEndpoint,
    };
    use abstract_testing::prelude::{TEST_ADMIN, TEST_ANS_HOST, TEST_VERSION_CONTROL};
    use cosmwasm_std::{
        testing::{mock_dependencies, mock_env, mock_info},
        SubMsgResult,
    };
    use speculoos::prelude::*;

    #[test]
    fn exports_endpoints() {
        export_endpoints!(MOCK_ADAPTER, MockAdapterContract);

        let mut deps = mock_dependencies();

        // init
        let init_msg = adapter::InstantiateMsg {
            base: adapter::BaseInstantiateMsg {
                ans_host_address: TEST_ANS_HOST.to_string(),
                version_control_address: TEST_VERSION_CONTROL.to_string(),
            },
            module: MockInitMsg,
        };
        let actual_init = instantiate(
            deps.as_mut(),
            mock_env(),
            mock_info(TEST_ADMIN, &[]),
            init_msg.clone(),
        );
        let expected_init = MOCK_ADAPTER.instantiate(
            deps.as_mut(),
            mock_env(),
            mock_info(TEST_ADMIN, &[]),
            init_msg,
        );
        assert_that!(actual_init).is_equal_to(expected_init);

        // exec
        let exec_msg = adapter::ExecuteMsg::Module(AdapterRequestMsg::new(None, MockExecMsg));
        let actual_exec = execute(
            deps.as_mut(),
            mock_env(),
            mock_info(TEST_ADMIN, &[]),
            exec_msg.clone(),
        );
        let expected_exec = MOCK_ADAPTER.execute(
            deps.as_mut(),
            mock_env(),
            mock_info(TEST_ADMIN, &[]),
            exec_msg,
        );
        assert_that!(actual_exec).is_equal_to(expected_exec);

        // query
        let query_msg = adapter::QueryMsg::Module(MockQueryMsg);
        let actual_query = query(deps.as_ref(), mock_env(), query_msg.clone());
        let expected_query = MOCK_ADAPTER.query(deps.as_ref(), mock_env(), query_msg);
        assert_that!(actual_query).is_equal_to(expected_query);

        // sudo
        let sudo_msg = MockSudoMsg {};
        let actual_sudo = sudo(deps.as_mut(), mock_env(), sudo_msg.clone());
        let expected_sudo = MOCK_ADAPTER.sudo(deps.as_mut(), mock_env(), sudo_msg);
        assert_that!(actual_sudo).is_equal_to(expected_sudo);

        // reply
        let reply_msg = ::cosmwasm_std::Reply {
            id: 0,
            result: SubMsgResult::Err("test".into()),
        };
        let actual_reply = reply(deps.as_mut(), mock_env(), reply_msg.clone());
        let expected_reply = MOCK_ADAPTER.reply(deps.as_mut(), mock_env(), reply_msg);
        assert_that!(actual_reply).is_equal_to(expected_reply);
    }
}
