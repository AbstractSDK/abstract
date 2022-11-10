#[macro_export]
macro_rules! export_endpoints {
    ($add_on_const:expr, $add_on_type:ty) => {
        use abstract_sdk::{
            ExecuteEndpoint, InstantiateEndpoint, MigrateEndpoint, QueryEndpoint, ReplyEndpoint,
        };

        /// Instantiate entrypoint
        #[cosmwasm_std::entry_point]
        pub fn instantiate(
            deps: cosmwasm_std::DepsMut,
            env: cosmwasm_std::Env,
            info: cosmwasm_std::MessageInfo,
            msg: <$add_on_type as abstract_sdk::InstantiateEndpoint>::InstantiateMsg,
        ) -> Result<cosmwasm_std::Response, <$add_on_type as abstract_sdk::Handler>::Error> {
            $add_on_const.instantiate(deps, env, info, msg)
        }

        /// Execute entrypoint
        #[cosmwasm_std::entry_point]
        pub fn execute(
            deps: cosmwasm_std::DepsMut,
            env: cosmwasm_std::Env,
            info: cosmwasm_std::MessageInfo,
            msg: <$add_on_type as abstract_sdk::ExecuteEndpoint>::ExecuteMsg,
        ) -> Result<cosmwasm_std::Response, <$add_on_type as abstract_sdk::Handler>::Error> {
            $add_on_const.execute(deps, env, info, msg)
        }

        /// Query entrypoint
        #[cosmwasm_std::entry_point]
        pub fn query(
            deps: cosmwasm_std::Deps,
            env: cosmwasm_std::Env,
            msg: <$add_on_type as abstract_sdk::QueryEndpoint>::QueryMsg,
        ) -> cosmwasm_std::StdResult<cosmwasm_std::Binary> {
            $add_on_const.query(deps, env, msg)
        }

        /// Migrate entrypoint
        #[cosmwasm_std::entry_point]
        pub fn migrate(
            deps: cosmwasm_std::DepsMut,
            env: cosmwasm_std::Env,
            msg: <$add_on_type as abstract_sdk::MigrateEndpoint>::MigrateMsg,
        ) -> Result<cosmwasm_std::Response, <$add_on_type as abstract_sdk::Handler>::Error> {
            $add_on_const.migrate(deps, env, msg)
        }

        // Reply entrypoint
        #[cosmwasm_std::entry_point]
        pub fn reply(
            deps: cosmwasm_std::DepsMut,
            env: cosmwasm_std::Env,
            msg: cosmwasm_std::Reply,
        ) -> Result<cosmwasm_std::Response, <$add_on_type as abstract_sdk::Handler>::Error> {
            $add_on_const.reply(deps, env, msg)
        }
    };
}
