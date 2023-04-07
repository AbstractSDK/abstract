mod execute;
mod ibc_callback;
pub mod instantiate;
mod migrate;
mod query;
mod receive;
mod reply;
mod sudo;

#[macro_export]
/// Exports all entrypoints
/// Disable export with "library" feature
macro_rules! export_endpoints {
    ($app_const:expr, $app_type:ty) => {
        /// Instantiate entrypoint
        #[::cosmwasm_std::entry_point]
        pub fn instantiate(
            deps: ::cosmwasm_std::DepsMut,
            env: ::cosmwasm_std::Env,
            info: ::cosmwasm_std::MessageInfo,
            msg: <$app_type as ::abstract_sdk::base::InstantiateEndpoint>::InstantiateMsg,
        ) -> Result<::cosmwasm_std::Response, <$app_type as ::abstract_sdk::base::Handler>::Error> {
            use ::abstract_sdk::base::InstantiateEndpoint;
            $app_const.instantiate(deps, env, info, msg)
        }

        /// Execute entrypoint
        #[::cosmwasm_std::entry_point]
        pub fn execute(
            deps: ::cosmwasm_std::DepsMut,
            env: ::cosmwasm_std::Env,
            info: ::cosmwasm_std::MessageInfo,
            msg: <$app_type as ::abstract_sdk::base::ExecuteEndpoint>::ExecuteMsg,
        ) -> Result<::cosmwasm_std::Response, <$app_type as ::abstract_sdk::base::Handler>::Error> {
            use ::abstract_sdk::base::ExecuteEndpoint;
            $app_const.execute(deps, env, info, msg)
        }

        /// Query entrypoint
        #[::cosmwasm_std::entry_point]
        pub fn query(
            deps: ::cosmwasm_std::Deps,
            env: ::cosmwasm_std::Env,
            msg: <$app_type as abstract_sdk::base::QueryEndpoint>::QueryMsg,
        ) -> Result<::cosmwasm_std::Binary, <$app_type as ::abstract_sdk::base::Handler>::Error> {
            use ::abstract_sdk::base::QueryEndpoint;
            $app_const.query(deps, env, msg)
        }

        /// Migrate entrypoint
        #[::cosmwasm_std::entry_point]
        pub fn migrate(
            deps: ::cosmwasm_std::DepsMut,
            env: ::cosmwasm_std::Env,
            msg: <$app_type as abstract_sdk::base::MigrateEndpoint>::MigrateMsg,
        ) -> Result<::cosmwasm_std::Response, <$app_type as ::abstract_sdk::base::Handler>::Error> {
            use ::abstract_sdk::base::MigrateEndpoint;
            $app_const.migrate(deps, env, msg)
        }

        // Reply entrypoint
        #[::cosmwasm_std::entry_point]
        pub fn reply(
            deps: ::cosmwasm_std::DepsMut,
            env: ::cosmwasm_std::Env,
            msg: ::cosmwasm_std::Reply,
        ) -> Result<::cosmwasm_std::Response, <$app_type as ::abstract_sdk::base::Handler>::Error> {
            use ::abstract_sdk::base::ReplyEndpoint;
            $app_const.reply(deps, env, msg)
        }

        // Sudo entrypoint
        #[::cosmwasm_std::entry_point]
        pub fn sudo(
            deps: ::cosmwasm_std::DepsMut,
            env: ::cosmwasm_std::Env,
            msg: <$app_type as ::abstract_sdk::base::Handler>::SudoMsg,
        ) -> Result<::cosmwasm_std::Response, <$app_type as ::abstract_sdk::base::Handler>::Error> {
            use ::abstract_sdk::base::SudoEndpoint;
            $app_const.sudo(deps, env, msg)
        }
    };
}

#[cfg(test)]
mod test {
    use abstract_sdk::base::{
        ExecuteEndpoint, InstantiateEndpoint, MigrateEndpoint, QueryEndpoint, ReplyEndpoint,
        SudoEndpoint,
    };
    use abstract_testing::prelude::{TEST_ADMIN, TEST_ANS_HOST};
    use cosmwasm_std::{
        testing::{mock_dependencies, mock_env, mock_info},
        SubMsgResult,
    };
    use speculoos::prelude::*;

    use crate::mock::*;

    #[test]
    fn exports_endpoints() {
        export_endpoints!(MOCK_APP, MockAppContract);

        let mut deps = mock_dependencies();

        // init
        let init_msg = app::InstantiateMsg {
            base: app::BaseInstantiateMsg {
                ans_host_address: TEST_ANS_HOST.to_string(),
            },
            module: MockInitMsg,
        };
        let actual_init = instantiate(
            deps.as_mut(),
            mock_env(),
            mock_info(TEST_ADMIN, &[]),
            init_msg.clone(),
        );
        let expected_init = MOCK_APP.instantiate(
            deps.as_mut(),
            mock_env(),
            mock_info(TEST_ADMIN, &[]),
            init_msg,
        );
        assert_that!(actual_init).is_equal_to(expected_init);

        // exec
        let exec_msg = app::ExecuteMsg::Module(MockExecMsg);
        let actual_exec = execute(
            deps.as_mut(),
            mock_env(),
            mock_info(TEST_ADMIN, &[]),
            exec_msg.clone(),
        );
        let expected_exec = MOCK_APP.execute(
            deps.as_mut(),
            mock_env(),
            mock_info(TEST_ADMIN, &[]),
            exec_msg,
        );
        assert_that!(actual_exec).is_equal_to(expected_exec);

        // query
        let query_msg = app::QueryMsg::Module(MockQueryMsg);
        let actual_query = query(deps.as_ref(), mock_env(), query_msg.clone());
        let expected_query = MOCK_APP.query(deps.as_ref(), mock_env(), query_msg);
        assert_that!(actual_query).is_equal_to(expected_query);

        // migrate
        let migrate_msg = app::MigrateMsg {
            base: app::BaseMigrateMsg {},
            module: MockMigrateMsg,
        };
        let actual_migrate = migrate(deps.as_mut(), mock_env(), migrate_msg.clone());
        let expected_migrate = MOCK_APP.migrate(deps.as_mut(), mock_env(), migrate_msg);
        assert_that!(actual_migrate).is_equal_to(expected_migrate);

        // sudo
        let sudo_msg = MockSudoMsg {};
        let actual_sudo = sudo(deps.as_mut(), mock_env(), sudo_msg.clone());
        let expected_sudo = MOCK_APP.sudo(deps.as_mut(), mock_env(), sudo_msg);
        assert_that!(actual_sudo).is_equal_to(expected_sudo);

        // reply
        let reply_msg = ::cosmwasm_std::Reply {
            id: 0,
            result: SubMsgResult::Err("test".into()),
        };
        let actual_reply = reply(deps.as_mut(), mock_env(), reply_msg.clone());
        let expected_reply = MOCK_APP.reply(deps.as_mut(), mock_env(), reply_msg);
        assert_that!(actual_reply).is_equal_to(expected_reply);
    }
}
