mod execute;
mod ibc_callback;
pub mod instantiate;
mod query;
mod receive;

#[macro_export]
macro_rules! export_endpoints {
    ($app_const:expr, $app_type:ty) => {
        /// Instantiate entrypoint
        #[cosmwasm_std::entry_point]
        pub fn instantiate(
            deps: cosmwasm_std::DepsMut,
            env: cosmwasm_std::Env,
            info: cosmwasm_std::MessageInfo,
            msg: <$app_type as abstract_sdk::base::InstantiateEndpoint>::InstantiateMsg,
        ) -> Result<cosmwasm_std::Response, <$app_type as abstract_sdk::base::Handler>::Error> {
            use abstract_sdk::base::InstantiateEndpoint;
            $app_const.instantiate(deps, env, info, msg)
        }

        /// Execute entrypoint
        #[cosmwasm_std::entry_point]
        pub fn execute(
            deps: cosmwasm_std::DepsMut,
            env: cosmwasm_std::Env,
            info: cosmwasm_std::MessageInfo,
            msg: <$app_type as abstract_sdk::base::ExecuteEndpoint>::ExecuteMsg,
        ) -> Result<cosmwasm_std::Response, <$app_type as abstract_sdk::base::Handler>::Error> {
            use abstract_sdk::base::ExecuteEndpoint;
            $app_const.execute(deps, env, info, msg)
        }

        /// Query entrypoint
        #[cosmwasm_std::entry_point]
        pub fn query(
            deps: cosmwasm_std::Deps,
            env: cosmwasm_std::Env,
            msg: <$app_type as abstract_sdk::base::QueryEndpoint>::QueryMsg,
        ) -> Result<cosmwasm_std::Binary, <$app_type as abstract_sdk::base::Handler>::Error> {
            use abstract_sdk::base::QueryEndpoint;
            $app_const.query(deps, env, msg)
        }
    };
}
