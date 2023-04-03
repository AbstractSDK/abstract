mod execute;
mod ibc_callback;
mod instantiate;
mod query;
mod receive;
mod reply;

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
    };
}
