#[macro_export]
/// Creates the interface for working with the app wth cw-orch
macro_rules! create_interface {
    ($app_const:expr, $app_type:ident) => {
    	mod _wrapper_fns{
    		use super::*;
	        pub fn instantiate(
	            deps: ::cosmwasm_std::DepsMut,
	            env: ::cosmwasm_std::Env,
	            info: ::cosmwasm_std::MessageInfo,
	            msg: <$app_type as ::abstract_sdk::base::InstantiateEndpoint>::InstantiateMsg,
	        ) -> Result<::cosmwasm_std::Response, <$app_type as ::abstract_sdk::base::Handler>::Error> {
	            use ::abstract_sdk::base::InstantiateEndpoint;
	            $app_const.instantiate(deps, env, info, msg)
	        }

	        pub fn execute(
	            deps: ::cosmwasm_std::DepsMut,
	            env: ::cosmwasm_std::Env,
	            info: ::cosmwasm_std::MessageInfo,
	            msg: <$app_type as ::abstract_sdk::base::ExecuteEndpoint>::ExecuteMsg,
	        ) -> Result<::cosmwasm_std::Response, <$app_type as ::abstract_sdk::base::Handler>::Error> {
	            use ::abstract_sdk::base::ExecuteEndpoint;
	            $app_const.execute(deps, env, info, msg)
	        }

	        pub fn query(
	            deps: ::cosmwasm_std::Deps,
	            env: ::cosmwasm_std::Env,
	            msg: <$app_type as abstract_sdk::base::QueryEndpoint>::QueryMsg,
	        ) -> Result<::cosmwasm_std::Binary, <$app_type as ::abstract_sdk::base::Handler>::Error> {
	            use ::abstract_sdk::base::QueryEndpoint;
	            $app_const.query(deps, env, msg)
	        }

	        pub fn migrate(
	            deps: ::cosmwasm_std::DepsMut,
	            env: ::cosmwasm_std::Env,
	            msg: <$app_type as abstract_sdk::base::MigrateEndpoint>::MigrateMsg,
	        ) -> Result<::cosmwasm_std::Response, <$app_type as ::abstract_sdk::base::Handler>::Error> {
	            use ::abstract_sdk::base::MigrateEndpoint;
	            $app_const.migrate(deps, env, msg)
	        }

	        pub fn reply(
	            deps: ::cosmwasm_std::DepsMut,
	            env: ::cosmwasm_std::Env,
	            msg: ::cosmwasm_std::Reply,
	        ) -> Result<::cosmwasm_std::Response, <$app_type as ::abstract_sdk::base::Handler>::Error> {
	            use ::abstract_sdk::base::ReplyEndpoint;
	            $app_const.reply(deps, env, msg)
	        }

	        pub fn sudo(
	            deps: ::cosmwasm_std::DepsMut,
	            env: ::cosmwasm_std::Env,
	            msg: <$app_type as ::abstract_sdk::base::Handler>::SudoMsg,
	        ) -> Result<::cosmwasm_std::Response, <$app_type as ::abstract_sdk::base::Handler>::Error> {
	            use ::abstract_sdk::base::SudoEndpoint;
	            $app_const.sudo(deps, env, msg)
	        }

	        pub type InstantiateMsg = <$app_type as ::abstract_sdk::base::InstantiateEndpoint>::InstantiateMsg;
	        pub type ExecuteMsg = <$app_type as ::abstract_sdk::base::ExecuteEndpoint>::ExecuteMsg;
	        pub type QueryMsg = <$app_type as ::abstract_sdk::base::QueryEndpoint>::QueryMsg;
	        pub type MigrateMsg = <$app_type as ::abstract_sdk::base::MigrateEndpoint>::MigrateMsg;
	    }

	    pub mod interface{
	    	use super::_wrapper_fns;
	    	#[::cw_orch::interface(_wrapper_fns::InstantiateMsg, _wrapper_fns::ExecuteMsg, _wrapper_fns::QueryMsg, _wrapper_fns::MigrateMsg)]
			pub struct $app_type;


			impl <Chain: ::cw_orch::prelude::CwEnv> ::cw_orch::prelude::Uploadable for $app_type<Chain> {
			    fn wasm(&self) -> ::cw_orch::prelude::WasmPath {
			    	let wasm_name = env!("CARGO_CRATE_NAME").replace('-', "_");
			        ::cw_orch::prelude::ArtifactsDir::auto(Some(env!("CARGO_MANIFEST_DIR").to_string()))
			        	.find_wasm_path(&wasm_name).unwrap()
			    }

			    fn wrapper(
			        &self,
			    ) -> Box<dyn ::cw_orch::prelude::MockContract<::cosmwasm_std::Empty, ::cosmwasm_std::Empty>> {
			        Box::new(
			            ::cw_orch::prelude::ContractWrapper::new_with_empty(
			                _wrapper_fns::execute,
			                _wrapper_fns::instantiate,
			                _wrapper_fns::query,
			            )
			            .with_reply(_wrapper_fns::reply)
			            .with_migrate(_wrapper_fns::migrate)
						.with_sudo(_wrapper_fns::sudo),
			        )
			    }
			}

			impl<Chain: ::cw_orch::prelude::CwEnv> ::abstract_interface::AppDeployer<Chain> for $app_type<Chain> {}
	    }


    };
}
