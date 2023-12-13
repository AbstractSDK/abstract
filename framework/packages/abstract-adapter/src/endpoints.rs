mod execute;
mod ibc_callback;
mod instantiate;
mod query;
mod receive;
mod reply;
mod sudo;

#[macro_export]
macro_rules! export_endpoints {
    ($api_func:expr, $api_type:ident) => {
        /// Instantiate entrypoint
        #[::cosmwasm_std::entry_point]
        pub fn instantiate<'a>(
            deps: ::cosmwasm_std::DepsMut<'a>,
            env: ::cosmwasm_std::Env,
            info: ::cosmwasm_std::MessageInfo,
            msg: <$api_type<'a> as ::abstract_sdk::base::InstantiateEndpoint>::InstantiateMsg,
        ) -> Result<
        ::cosmwasm_std::Response,
            <$api_type<'a> as ::abstract_sdk::base::Handler>::Error,
        > {
            use ::abstract_sdk::base::InstantiateEndpoint;
            let ctx = (deps, env, info).into();
            let api = $api_func(ctx);
            api.instantiate(msg)
        }

        /// Execute entrypoint
        #[::cosmwasm_std::entry_point]
        pub fn execute<'a>(
            deps: ::cosmwasm_std::DepsMut<'a>,
            env: ::cosmwasm_std::Env,
            info: ::cosmwasm_std::MessageInfo,
            msg: <$api_type<'a> as ::abstract_sdk::base::ExecuteEndpoint>::ExecuteMsg,
        ) -> Result<
        ::cosmwasm_std::Response,
            <$api_type<'a> as ::abstract_sdk::base::Handler>::Error,
        > {
            use ::abstract_sdk::base::ExecuteEndpoint;
            let ctx = (deps, env, info).into();
            let api = $api_func(ctx);
            api.execute(msg)
        }

        /// Query entrypoint
        #[::cosmwasm_std::entry_point]
        pub fn query<'a>(
            deps: ::cosmwasm_std::Deps<'a>,
            env: ::cosmwasm_std::Env,
            msg: <$api_type<'a> as ::abstract_sdk::base::QueryEndpoint>::QueryMsg,
        ) -> Result<::cosmwasm_std::Binary, <$api_type<'a> as ::abstract_sdk::base::Handler>::Error> {
            use ::abstract_sdk::base::QueryEndpoint;
            let ctx = (deps, env).into();
            let api = $api_func(ctx);
            api.query(msg)
        }

        // Reply entrypoint
        #[::cosmwasm_std::entry_point]
        pub fn reply(
            deps: ::cosmwasm_std::DepsMut,
            env: ::cosmwasm_std::Env,
            msg: ::cosmwasm_std::Reply,
        ) -> Result<::cosmwasm_std::Response, <$api_type as ::abstract_sdk::base::Handler>::Error> {
            use ::abstract_sdk::base::ReplyEndpoint;
            let ctx = (deps, env).into();
            let api = $api_func(ctx);
            api.reply(msg)
        }

        // Sudo entrypoint
        #[::cosmwasm_std::entry_point]
        pub fn sudo<'a>(
            deps: ::cosmwasm_std::DepsMut<'a>,
            env: ::cosmwasm_std::Env,
            msg: <$api_type<'a> as ::abstract_sdk::base::Handler>::SudoMsg,
        ) -> Result<::cosmwasm_std::Response, <$api_type<'a> as ::abstract_sdk::base::Handler>::Error> {
            use ::abstract_sdk::base::SudoEndpoint;
            let ctx = (deps, env).into();
            let api = $api_func(ctx);
            api.sudo( msg)
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
    use abstract_testing::prelude::*;
    use cosmwasm_std::{
        testing::{mock_dependencies, mock_env, mock_info},
        SubMsgResult,
    };
    use speculoos::prelude::*;

    #[test]
    fn exports_endpoints() {
        export_endpoints!(mock_adapter, MockAdapterContract);

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
            mock_info(OWNER, &[]),
            init_msg.clone(),
        );
        let expected_init = mock_adapter((deps.as_mut(), mock_env(), mock_info(OWNER, &[])).into())
            .instantiate(init_msg);
        assert_that!(actual_init).is_equal_to(expected_init);

        // exec
        let exec_msg = adapter::ExecuteMsg::Module(AdapterRequestMsg::new(None, MockExecMsg));
        let actual_exec = execute(
            deps.as_mut(),
            mock_env(),
            mock_info(OWNER, &[]),
            exec_msg.clone(),
        );
        let expected_exec = mock_adapter((deps.as_mut(), mock_env(), mock_info(OWNER, &[])).into())
            .execute(exec_msg);
        assert_that!(actual_exec).is_equal_to(expected_exec);

        // query
        let query_msg = adapter::QueryMsg::Module(MockQueryMsg);
        let actual_query = query(deps.as_ref(), mock_env(), query_msg.clone());
        let expected_query = mock_adapter((deps.as_ref(), mock_env()).into()).query(query_msg);
        assert_that!(actual_query).is_equal_to(expected_query);

        // sudo
        let sudo_msg = MockSudoMsg {};
        let actual_sudo = sudo(deps.as_mut(), mock_env(), sudo_msg.clone());
        let expected_sudo = mock_adapter((deps.as_mut(), mock_env()).into()).sudo(sudo_msg);
        assert_that!(actual_sudo).is_equal_to(expected_sudo);

        // reply
        let reply_msg = ::cosmwasm_std::Reply {
            id: 0,
            result: SubMsgResult::Err("test".into()),
        };
        let actual_reply = reply(deps.as_mut(), mock_env(), reply_msg.clone());
        let expected_reply = mock_adapter((deps.as_mut(), mock_env()).into()).reply(reply_msg);
        assert_that!(actual_reply).is_equal_to(expected_reply);
    }
}
