mod execute;
mod ibc_callback;
pub mod instantiate;
mod migrate;
mod query;
mod receive;
mod reply;
mod sudo;

#[macro_export]
/// Exports all entry-points, should be enabled by default.
/// - instantiate
/// - execute
/// - query
/// - migrate
/// - reply
/// - sudo
///
/// ## Usage
/// Requires two arguments:
/// 1. The App constant.
/// 2. The App type.
///
/// ```ignore
/// abstract_app::export_endpoints!(MY_APP, MyApp);
/// ```
macro_rules! export_endpoints {
    ($app_const:expr, $app_type:ident) => {
        /// Instantiate entrypoint
        #[::cosmwasm_std::entry_point]
        pub fn instantiate<'a>(
            deps: ::cosmwasm_std::DepsMut<'a>,
            env: ::cosmwasm_std::Env,
            info: ::cosmwasm_std::MessageInfo,
            msg: <$app_type<'a> as ::abstract_sdk::base::InstantiateEndpoint>::InstantiateMsg,
        ) -> Result<
            ::cosmwasm_std::Response,
            <$app_type<'a> as ::abstract_sdk::base::Handler>::Error,
        > {
            use ::abstract_sdk::base::InstantiateEndpoint;
            let ctx = (deps, env, info).into();
            let app = $app_const(ctx);
            app.instantiate(msg)
        }

        /// Execute entrypoint
        #[::cosmwasm_std::entry_point]
        pub fn execute<'a>(
            deps: ::cosmwasm_std::DepsMut<'a>,
            env: ::cosmwasm_std::Env,
            info: ::cosmwasm_std::MessageInfo,
            msg: <$app_type as ::abstract_sdk::base::ExecuteEndpoint>::ExecuteMsg,
        ) -> Result<::cosmwasm_std::Response, <$app_type<'a> as ::abstract_sdk::base::Handler>::Error> {
            use ::abstract_sdk::base::ExecuteEndpoint;
            let ctx = (deps, env, info).into();
            let app = $app_const(ctx);
            app.execute(msg)
        }

        /// Query entrypoint
        #[::cosmwasm_std::entry_point]
        pub fn query<'a>(
            deps: ::cosmwasm_std::Deps,
            env: ::cosmwasm_std::Env,
            msg: <$app_type as abstract_sdk::base::QueryEndpoint>::QueryMsg,
        ) -> Result<::cosmwasm_std::Binary, <$app_type<'a> as ::abstract_sdk::base::Handler>::Error> {
            use ::abstract_sdk::base::QueryEndpoint;
            let ctx = (deps, env).into();
            let app = $app_const(ctx);
            app.query(msg)
        }

        /// Migrate entrypoint
        #[::cosmwasm_std::entry_point]
        pub fn migrate<'a>(
            deps: ::cosmwasm_std::DepsMut,
            env: ::cosmwasm_std::Env,
            msg: <$app_type as abstract_sdk::base::MigrateEndpoint>::MigrateMsg,
        ) -> Result<::cosmwasm_std::Response, <$app_type<'a> as ::abstract_sdk::base::Handler>::Error> {
            use ::abstract_sdk::base::MigrateEndpoint;
            let ctx = (deps, env).into();
            let app = $app_const(ctx);
            app.migrate(msg)
        }

        // Reply entrypoint
        #[::cosmwasm_std::entry_point]
        pub fn reply<'a>(
            deps: ::cosmwasm_std::DepsMut,
            env: ::cosmwasm_std::Env,
            msg: ::cosmwasm_std::Reply,
        ) -> Result<::cosmwasm_std::Response, <$app_type<'a> as ::abstract_sdk::base::Handler>::Error> {
            use ::abstract_sdk::base::ReplyEndpoint;
            let ctx = (deps, env).into();
            let app = $app_const(ctx);
            app.reply(msg)
        }

        // Sudo entrypoint
        #[::cosmwasm_std::entry_point]
        pub fn sudo<'a>(
            deps: ::cosmwasm_std::DepsMut,
            env: ::cosmwasm_std::Env,
            msg: <$app_type as ::abstract_sdk::base::Handler>::SudoMsg,
        ) -> Result<::cosmwasm_std::Response, <$app_type<'a> as ::abstract_sdk::base::Handler>::Error> {
            use ::abstract_sdk::base::SudoEndpoint;
            let ctx = (deps, env).into();
            let app = $app_const(ctx);
            app.sudo(msg)
        }
    };
}

#[cfg(test)]
mod test {
    use abstract_sdk::base::{
        ExecuteEndpoint, InstantiateEndpoint, MigrateEndpoint, QueryEndpoint, ReplyEndpoint,
        SudoEndpoint,
    };
    use abstract_testing::{addresses::test_account_base, prelude::*};
    use cosmwasm_std::{
        testing::{mock_dependencies, mock_env, mock_info},
        SubMsgResult,
    };
    use speculoos::prelude::*;

    use crate::mock::*;

    #[test]
    fn exports_endpoints() {
        export_endpoints!(mock_app, MockAppContract);

        let mut deps = mock_dependencies();

        // init
        let init_msg = app::InstantiateMsg {
            base: app::BaseInstantiateMsg {
                ans_host_address: TEST_ANS_HOST.to_string(),
                version_control_address: TEST_VERSION_CONTROL.to_string(),
                account_base: test_account_base(),
            },
            module: MockInitMsg,
        };
        let actual_init = instantiate(
            deps.as_mut(),
            mock_env(),
            mock_info(OWNER, &[]),
            init_msg.clone(),
        );
        let expected_init = mock_app((deps.as_mut(), mock_env(), mock_info(OWNER, &[])).into())
            .instantiate(init_msg);
        assert_that!(actual_init).is_equal_to(expected_init);

        // exec
        let exec_msg = app::ExecuteMsg::Module(MockExecMsg::DoSomething {});
        let actual_exec = mock_app((deps.as_mut(), mock_env(), mock_info(OWNER, &[])).into())
            .execute(exec_msg.clone());
        let expected_exec = execute(deps.as_mut(), mock_env(), mock_info(OWNER, &[]), exec_msg);
        assert_that!(actual_exec).is_equal_to(expected_exec);

        // query
        let query_msg = app::QueryMsg::Module(MockQueryMsg::GetSomething {});
        let actual_query = query(deps.as_ref(), mock_env(), query_msg.clone());
        let expected_query = mock_app((deps.as_ref(), mock_env()).into()).query(query_msg);
        assert_that!(actual_query).is_equal_to(expected_query);

        // migrate
        let migrate_msg = app::MigrateMsg {
            base: app::BaseMigrateMsg {},
            module: MockMigrateMsg,
        };
        let actual_migrate = migrate(deps.as_mut(), mock_env(), migrate_msg.clone());
        let expected_migrate = mock_app((deps.as_mut(), mock_env()).into()).migrate(migrate_msg);
        assert_that!(actual_migrate).is_equal_to(expected_migrate);

        // sudo
        let sudo_msg = MockSudoMsg {};
        let actual_sudo = sudo(deps.as_mut(), mock_env(), sudo_msg.clone());
        let expected_sudo = mock_app((deps.as_mut(), mock_env()).into()).sudo(sudo_msg);
        assert_that!(actual_sudo).is_equal_to(expected_sudo);

        // reply
        let reply_msg = ::cosmwasm_std::Reply {
            id: 0,
            result: SubMsgResult::Err("test".into()),
        };
        let actual_reply = reply(deps.as_mut(), mock_env(), reply_msg.clone());
        let expected_reply = mock_app((deps.as_mut(), mock_env()).into()).reply(reply_msg);
        assert_that!(actual_reply).is_equal_to(expected_reply);
    }
}
