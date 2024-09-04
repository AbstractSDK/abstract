mod execute;
mod ibc_callback;
pub mod instantiate;
mod migrate;
mod module_ibc;
mod query;
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
    ($app_const:expr, $app_type:ty) => {
        $crate::__endpoints_without_custom__!($app_const, $app_type);

        /// Execute entrypoint
        #[::cosmwasm_std::entry_point]
        pub fn execute(
            deps: ::cosmwasm_std::DepsMut,
            env: ::cosmwasm_std::Env,
            info: ::cosmwasm_std::MessageInfo,
            msg: <$app_type as $crate::sdk::base::ExecuteEndpoint>::ExecuteMsg,
        ) -> Result<::cosmwasm_std::Response, <$app_type as $crate::sdk::base::Handler>::Error> {
            use $crate::sdk::base::ExecuteEndpoint;
            $app_const.execute(deps, env, info, msg)
        }
    };
    ($app_const:expr, $app_type:ty, $custom_exec:ty) => {
        $crate::__endpoints_without_custom__!($app_const, $app_type);

        /// Execute entrypoint
        #[::cosmwasm_std::entry_point]
        pub fn execute(
            deps: ::cosmwasm_std::DepsMut,
            env: ::cosmwasm_std::Env,
            info: ::cosmwasm_std::MessageInfo,
            msg: $custom_exec,
        ) -> Result<::cosmwasm_std::Response, <$app_type as $crate::sdk::base::Handler>::Error> {
            use $crate::sdk::base::{CustomExecuteHandler, ExecuteEndpoint};
            match CustomExecuteHandler::try_into_base(msg) {
                Ok(default) => $app_const.execute(deps, env, info, default),
                Err(custom) => custom.custom_execute(deps, env, info, $app_const),
            }
        }
    };
}

#[macro_export]
#[doc(hidden)]
macro_rules! __endpoints_without_custom__ {
    ($app_const:expr, $app_type:ty) => {
        /// Instantiate entrypoint
        #[::cosmwasm_std::entry_point]
        pub fn instantiate(
            deps: ::cosmwasm_std::DepsMut,
            env: ::cosmwasm_std::Env,
            info: ::cosmwasm_std::MessageInfo,
            msg: <$app_type as $crate::sdk::base::InstantiateEndpoint>::InstantiateMsg,
        ) -> Result<::cosmwasm_std::Response, <$app_type as $crate::sdk::base::Handler>::Error> {
            use $crate::sdk::base::InstantiateEndpoint;
            $app_const.instantiate(deps, env, info, msg)
        }

        /// Query entrypoint
        #[::cosmwasm_std::entry_point]
        pub fn query(
            deps: ::cosmwasm_std::Deps,
            env: ::cosmwasm_std::Env,
            msg: <$app_type as $crate::sdk::base::QueryEndpoint>::QueryMsg,
        ) -> Result<::cosmwasm_std::Binary, <$app_type as $crate::sdk::base::Handler>::Error> {
            use $crate::sdk::base::QueryEndpoint;
            $app_const.query(deps, env, msg)
        }

        /// Migrate entrypoint
        #[::cosmwasm_std::entry_point]
        pub fn migrate(
            deps: ::cosmwasm_std::DepsMut,
            env: ::cosmwasm_std::Env,
            msg: <$app_type as $crate::sdk::base::MigrateEndpoint>::MigrateMsg,
        ) -> Result<::cosmwasm_std::Response, <$app_type as $crate::sdk::base::Handler>::Error> {
            use $crate::sdk::base::MigrateEndpoint;
            $app_const.migrate(deps, env, msg)
        }

        // Reply entrypoint
        #[::cosmwasm_std::entry_point]
        pub fn reply(
            deps: ::cosmwasm_std::DepsMut,
            env: ::cosmwasm_std::Env,
            msg: ::cosmwasm_std::Reply,
        ) -> Result<::cosmwasm_std::Response, <$app_type as $crate::sdk::base::Handler>::Error> {
            use $crate::sdk::base::ReplyEndpoint;
            $app_const.reply(deps, env, msg)
        }

        // Sudo entrypoint
        #[::cosmwasm_std::entry_point]
        pub fn sudo(
            deps: ::cosmwasm_std::DepsMut,
            env: ::cosmwasm_std::Env,
            msg: <$app_type as $crate::sdk::base::Handler>::SudoMsg,
        ) -> Result<::cosmwasm_std::Response, <$app_type as $crate::sdk::base::Handler>::Error> {
            use $crate::sdk::base::SudoEndpoint;
            $app_const.sudo(deps, env, msg)
        }
    };
}

#[cfg(test)]
mod test {
    #![allow(clippy::needless_borrows_for_generic_args)]
    use crate::mock::*;
    use crate::sdk::base::{
        ExecuteEndpoint, InstantiateEndpoint, MigrateEndpoint, QueryEndpoint, ReplyEndpoint,
        SudoEndpoint,
    };
    use abstract_sdk::base::CustomExecuteHandler;
    use abstract_testing::prelude::*;
    use cosmwasm_std::{Binary, SubMsgResult};
    use speculoos::prelude::*;

    #[test]
    fn exports_endpoints() {
        export_endpoints!(MOCK_APP_WITH_DEP, MockAppContract);

        let mut deps = mock_dependencies();
        let abstr = AbstractMockAddrs::new(deps.api);

        // init
        let init_msg = app::InstantiateMsg {
            base: app::BaseInstantiateMsg {
                ans_host_address: abstr.ans_host.to_string(),
                version_control_address: abstr.version_control.to_string(),
                account_base: abstr.account,
            },
            module: MockInitMsg {},
        };
        let actual_init = instantiate(
            deps.as_mut(),
            mock_env(),
            message_info(&abstr.owner, &[]),
            init_msg.clone(),
        );
        let expected_init = MOCK_APP_WITH_DEP.instantiate(
            deps.as_mut(),
            mock_env(),
            message_info(&abstr.owner, &[]),
            init_msg,
        );
        assert_that!(actual_init).is_equal_to(expected_init);

        // exec
        let exec_msg = app::ExecuteMsg::Module(MockExecMsg::DoSomething {});
        let actual_exec = execute(
            deps.as_mut(),
            mock_env(),
            message_info(&abstr.owner, &[]),
            exec_msg.clone(),
        );
        let expected_exec = MOCK_APP_WITH_DEP.execute(
            deps.as_mut(),
            mock_env(),
            message_info(&abstr.owner, &[]),
            exec_msg,
        );
        assert_that!(actual_exec).is_equal_to(expected_exec);

        // query
        let query_msg = app::QueryMsg::Module(MockQueryMsg::GetSomething {});
        let actual_query = query(deps.as_ref(), mock_env(), query_msg.clone());
        let expected_query = MOCK_APP_WITH_DEP.query(deps.as_ref(), mock_env(), query_msg);
        assert_that!(actual_query).is_equal_to(expected_query);

        // migrate
        let migrate_msg = app::MigrateMsg {
            base: app::BaseMigrateMsg {},
            module: MockMigrateMsg,
        };
        let actual_migrate = migrate(deps.as_mut(), mock_env(), migrate_msg.clone());
        let expected_migrate = MOCK_APP_WITH_DEP.migrate(deps.as_mut(), mock_env(), migrate_msg);
        assert_that!(actual_migrate).is_equal_to(expected_migrate);

        // sudo
        let sudo_msg = MockSudoMsg {};
        let actual_sudo = sudo(deps.as_mut(), mock_env(), sudo_msg.clone());
        let expected_sudo = MOCK_APP_WITH_DEP.sudo(deps.as_mut(), mock_env(), sudo_msg);
        assert_that!(actual_sudo).is_equal_to(expected_sudo);

        // reply
        let reply_msg = ::cosmwasm_std::Reply {
            id: 0,
            result: SubMsgResult::Err("test".into()),
            payload: Binary::default(),
            gas_used: 0,
        };
        let actual_reply = reply(deps.as_mut(), mock_env(), reply_msg.clone());
        let expected_reply = MOCK_APP_WITH_DEP.reply(deps.as_mut(), mock_env(), reply_msg);
        assert_that!(actual_reply).is_equal_to(expected_reply);
    }

    #[test]
    fn exports_endpoints_custom() {
        #[cosmwasm_schema::cw_serde]
        #[derive(cw_orch::ExecuteFns)]
        pub enum CustomExecMsg {
            Base(abstract_std::app::BaseExecuteMsg),
            Module(crate::mock::MockExecMsg),
            IbcCallback(abstract_std::ibc::IbcResponseMsg),
            ModuleIbc(abstract_std::ibc::ModuleIbcMsg),
            Foo {},
        }

        impl From<crate::mock::MockExecMsg> for CustomExecMsg {
            fn from(request: crate::mock::MockExecMsg) -> Self {
                Self::Module(request)
            }
        }

        impl CustomExecuteHandler<MockAppContract> for CustomExecMsg {
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
                _module: MockAppContract,
            ) -> Result<cosmwasm_std::Response, crate::mock::MockError> {
                Ok(cosmwasm_std::Response::new().set_data(b"foo"))
            }
        }

        export_endpoints!(MOCK_APP_WITH_DEP, MockAppContract, CustomExecMsg);

        let mut deps = mock_dependencies();

        // custom
        let actual_custom_exec = execute(
            deps.as_mut(),
            mock_env(),
            mock_info(OWNER, &[]),
            CustomExecMsg::Foo {},
        )
        .unwrap();
        let expected_custom_exec =
            cosmwasm_std::Response::<cosmwasm_std::Empty>::new().set_data(b"foo");
        assert_eq!(actual_custom_exec, expected_custom_exec);

        // Ensure nothing broken

        // init
        let init_msg = app::InstantiateMsg {
            base: app::BaseInstantiateMsg {
                ans_host_address: TEST_ANS_HOST.to_string(),
                version_control_address: TEST_VERSION_CONTROL.to_string(),
                account_base: test_account_base(),
            },
            module: MockInitMsg {},
        };
        let actual_init = instantiate(
            deps.as_mut(),
            mock_env(),
            mock_info(OWNER, &[]),
            init_msg.clone(),
        );
        let expected_init = MOCK_APP_WITH_DEP.instantiate(
            deps.as_mut(),
            mock_env(),
            mock_info(OWNER, &[]),
            init_msg,
        );
        assert_that!(actual_init).is_equal_to(expected_init);

        // exec
        let exec_msg = app::ExecuteMsg::Module(MockExecMsg::DoSomething {});
        let actual_exec = execute(
            deps.as_mut(),
            mock_env(),
            mock_info(OWNER, &[]),
            CustomExecMsg::Module(MockExecMsg::DoSomething {}),
        );
        let expected_exec =
            MOCK_APP_WITH_DEP.execute(deps.as_mut(), mock_env(), mock_info(OWNER, &[]), exec_msg);
        assert_that!(actual_exec).is_equal_to(expected_exec);

        // query
        let query_msg = app::QueryMsg::Module(MockQueryMsg::GetSomething {});
        let actual_query = query(deps.as_ref(), mock_env(), query_msg.clone());
        let expected_query = MOCK_APP_WITH_DEP.query(deps.as_ref(), mock_env(), query_msg);
        assert_that!(actual_query).is_equal_to(expected_query);

        // migrate
        let migrate_msg = app::MigrateMsg {
            base: app::BaseMigrateMsg {},
            module: MockMigrateMsg,
        };
        let actual_migrate = migrate(deps.as_mut(), mock_env(), migrate_msg.clone());
        let expected_migrate = MOCK_APP_WITH_DEP.migrate(deps.as_mut(), mock_env(), migrate_msg);
        assert_that!(actual_migrate).is_equal_to(expected_migrate);

        // sudo
        let sudo_msg = MockSudoMsg {};
        let actual_sudo = sudo(deps.as_mut(), mock_env(), sudo_msg.clone());
        let expected_sudo = MOCK_APP_WITH_DEP.sudo(deps.as_mut(), mock_env(), sudo_msg);
        assert_that!(actual_sudo).is_equal_to(expected_sudo);

        // reply
        let reply_msg = ::cosmwasm_std::Reply {
            id: 0,
            result: SubMsgResult::Err("test".into()),
        };
        let actual_reply = reply(deps.as_mut(), mock_env(), reply_msg.clone());
        let expected_reply = MOCK_APP_WITH_DEP.reply(deps.as_mut(), mock_env(), reply_msg);
        assert_that!(actual_reply).is_equal_to(expected_reply);
    }
}
