mod execute;
mod ibc_callback;
pub mod instantiate;
mod migrate;
mod query;
mod receive;
mod reply;

#[macro_export]
/// Exports all entrypoints
/// Disable export with "library" feature
macro_rules! export_endpoints {
    ($app_const:expr, $app_type:ty) => {
        /// Instantiate entrypoint
        #[cfg_attr(not(feature = "library"), ::cosmwasm_std::entry_point)]
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
        #[cfg_attr(not(feature = "library"), ::cosmwasm_std::entry_point)]
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
        #[cfg_attr(not(feature = "library"), ::cosmwasm_std::entry_point)]
        pub fn query(
            deps: ::cosmwasm_std::Deps,
            env: ::cosmwasm_std::Env,
            msg: <$app_type as abstract_sdk::base::QueryEndpoint>::QueryMsg,
        ) -> ::cosmwasm_std::StdResult<::cosmwasm_std::Binary> {
            use ::abstract_sdk::base::QueryEndpoint;
            $app_const.query(deps, env, msg)
        }

        /// Migrate entrypoint
        #[cfg_attr(not(feature = "library"), ::cosmwasm_std::entry_point)]
        pub fn migrate(
            deps: ::cosmwasm_std::DepsMut,
            env: ::cosmwasm_std::Env,
            msg: <$app_type as abstract_sdk::base::MigrateEndpoint>::MigrateMsg,
        ) -> Result<::cosmwasm_std::Response, <$app_type as ::abstract_sdk::base::Handler>::Error> {
            use ::abstract_sdk::base::MigrateEndpoint;
            $app_const.migrate(deps, env, msg)
        }

        // Reply entrypoint
        #[cfg_attr(not(feature = "library"), ::cosmwasm_std::entry_point)]
        pub fn reply(
            deps: ::cosmwasm_std::DepsMut,
            env: ::cosmwasm_std::Env,
            msg: ::cosmwasm_std::Reply,
        ) -> Result<::cosmwasm_std::Response, <$app_type as ::abstract_sdk::base::Handler>::Error> {
            use ::abstract_sdk::base::ReplyEndpoint;
            $app_const.reply(deps, env, msg)
        }
    };
}
