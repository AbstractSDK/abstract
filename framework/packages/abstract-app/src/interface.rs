#[macro_export]
/// Creates the interface for working with the app with [cw-orch](https://github.com/AbstractSDK/cw-orchestrator).
/// This generates all the necessary code used to interact with the app in an cw-orch environment
///
/// ## Usage
/// The macro takes three arguments:
/// 1. The app's constant, declared in `contract.rs`.
/// 2. The app's type, declared in `contract.rs`.
/// 3. The name of the interface struct to be generated.
/// ```rust,ignore
/// cw_orch_interface!(APP, App, AppInterface);
/// ```
///
/// This will generate :
/// ```rust,ignore
/// pub mod interface{
///     #[cw_orch::interface(App::InstantiateMsg, App::ExecuteMsg, App::QueryMsg, App::MigrateMsg)]
///     pub struct AppInterface;
///
///     impl <Chain: cw_orch::prelude::CwEnv> cw_orch::prelude::Uploadable for AppInterface<Chain> {
///            // Looks for the wasm file in the app's artifacts directory
///            // The name of the wasm file should contain the app crate name (snake_cased)
///         fn wasm(&self) -> cw_orch::prelude::WasmPath {
///             let wasm_name = env!("CARGO_CRATE_NAME").replace('-', "_");
///             cw_orch::prelude::ArtifactsDir::auto(Some(env!("CARGO_MANIFEST_DIR").to_string()))
///                 .find_wasm_path(&wasm_name).unwrap()
///         }
///
///         fn wrapper(
///             &self,
///         ) -> Box<dyn cw_orch::prelude::MockContract<cosmwasm_std::Empty, cosmwasm_std::Empty>> {
///             Box::new(
///                 cw_orch::prelude::ContractWrapper::new_with_empty(
///                     APP::execute, // This notation, doesn't actually work like so, but we use that to illustrate
///                     APP::instantiate,
///                     APP::query,
///                 )
///                 .with_reply(APP::reply)
///                 .with_migrate(APP::migrate)
///                 .with_sudo(APP::sudo),
///             )
///         }
///     }
///     impl<Chain: ::cw_orch::prelude::CwEnv> $crate::abstract_app::abstract_interface::AppDeployer<Chain> for AppInterface<Chain> {}
/// }
/// ```
macro_rules! cw_orch_interface {
    ($app_const:expr, $app_type:ty, $interface_name: ident) => {
        #[cfg(not(target_arch = "wasm32"))]
        mod _wrapper_fns {
            use super::*;
            $crate::__wrapper_fns_without_custom__!($app_const, $app_type);

            pub fn execute(
                deps: ::cosmwasm_std::DepsMut,
                env: ::cosmwasm_std::Env,
                info: ::cosmwasm_std::MessageInfo,
                msg: <$app_type as $crate::sdk::base::ExecuteEndpoint>::ExecuteMsg,
            ) -> Result<::cosmwasm_std::Response, <$app_type as $crate::sdk::base::Handler>::Error>
            {
                use $crate::sdk::base::ExecuteEndpoint;
                $app_const.execute(deps, env, info, msg)
            }
        }

        pub mod interface {
            use super::*;

            #[::cw_orch::interface(
                _wrapper_fns::InstantiateMsg,
                _wrapper_fns::ExecuteMsg,
                _wrapper_fns::QueryMsg,
                _wrapper_fns::MigrateMsg
            )]
            pub struct $interface_name;

            $crate::__cw_orch_interfaces__!($interface_name);

            #[cfg(not(target_arch = "wasm32"))]
            impl<Chain: ::cw_orch::prelude::CwEnv> $crate::abstract_interface::RegisteredModule
                for $interface_name<Chain>
            {
                type InitMsg = <$app_type as $crate::sdk::base::Handler>::CustomInitMsg;

                fn module_id<'a>() -> &'a str {
                    $app_const.module_id()
                }

                fn module_version<'a>() -> &'a str {
                    $app_const.version()
                }

                fn dependencies<'a>() -> &'a [$crate::std::objects::dependency::StaticDependency] {
                    $crate::sdk::base::Handler::dependencies(&$app_const)
                }
            }
        }
    };
    ($app_const:expr, $app_type:ty, $interface_name: ident, $custom_exec:ty) => {
        #[cfg(not(target_arch = "wasm32"))]
        mod _wrapper_fns {
            use super::*;
            $crate::__wrapper_fns_without_custom__!($app_const, $app_type);

            pub fn execute(
                deps: ::cosmwasm_std::DepsMut,
                env: ::cosmwasm_std::Env,
                info: ::cosmwasm_std::MessageInfo,
                msg: $custom_exec,
            ) -> Result<::cosmwasm_std::Response, <$app_type as $crate::sdk::base::Handler>::Error>
            {
                use $crate::sdk::base::{CustomExecuteHandler, ExecuteEndpoint};
                match CustomExecuteHandler::try_into_base(msg) {
                    Ok(default) => $app_const.execute(deps, env, info, default),
                    Err(custom) => custom.custom_execute(deps, env, info, $app_const),
                }
            }
        }

        pub mod interface {
            use super::*;

            #[::cw_orch::interface(
                _wrapper_fns::InstantiateMsg,
                $custom_exec,
                _wrapper_fns::QueryMsg,
                _wrapper_fns::MigrateMsg
            )]
            pub struct $interface_name;

            $crate::__cw_orch_interfaces__!($interface_name);

            #[cfg(not(target_arch = "wasm32"))]
            impl<Chain: ::cw_orch::prelude::CwEnv> $crate::abstract_interface::RegisteredModule
                for $interface_name<Chain>
            {
                type InitMsg = <$app_type as $crate::sdk::base::Handler>::CustomInitMsg;

                fn module_id<'a>() -> &'a str {
                    $app_const.module_id()
                }

                fn module_version<'a>() -> &'a str {
                    $app_const.version()
                }

                fn dependencies<'a>() -> &'a [$crate::std::objects::dependency::StaticDependency] {
                    $crate::sdk::base::Handler::dependencies(&$app_const)
                }
            }
        }
    };
}

#[macro_export]
#[doc(hidden)]
macro_rules! __wrapper_fns_without_custom__ {
    ($app_const:expr, $app_type:ty) => {
        pub fn instantiate(
            deps: ::cosmwasm_std::DepsMut,
            env: ::cosmwasm_std::Env,
            info: ::cosmwasm_std::MessageInfo,
            msg: <$app_type as $crate::sdk::base::InstantiateEndpoint>::InstantiateMsg,
        ) -> Result<::cosmwasm_std::Response, <$app_type as $crate::sdk::base::Handler>::Error> {
            use $crate::sdk::base::InstantiateEndpoint;
            $app_const.instantiate(deps, env, info, msg)
        }

        pub fn query(
            deps: ::cosmwasm_std::Deps,
            env: ::cosmwasm_std::Env,
            msg: <$app_type as $crate::sdk::base::QueryEndpoint>::QueryMsg,
        ) -> Result<::cosmwasm_std::Binary, <$app_type as $crate::sdk::base::Handler>::Error> {
            use $crate::sdk::base::QueryEndpoint;
            $app_const.query(deps, env, msg)
        }

        pub fn migrate(
            deps: ::cosmwasm_std::DepsMut,
            env: ::cosmwasm_std::Env,
            msg: <$app_type as $crate::sdk::base::MigrateEndpoint>::MigrateMsg,
        ) -> Result<::cosmwasm_std::Response, <$app_type as $crate::sdk::base::Handler>::Error> {
            use $crate::sdk::base::MigrateEndpoint;
            $app_const.migrate(deps, env, msg)
        }

        pub fn reply(
            deps: ::cosmwasm_std::DepsMut,
            env: ::cosmwasm_std::Env,
            msg: ::cosmwasm_std::Reply,
        ) -> Result<::cosmwasm_std::Response, <$app_type as $crate::sdk::base::Handler>::Error> {
            use $crate::sdk::base::ReplyEndpoint;
            $app_const.reply(deps, env, msg)
        }

        pub fn sudo(
            deps: ::cosmwasm_std::DepsMut,
            env: ::cosmwasm_std::Env,
            msg: <$app_type as $crate::sdk::base::Handler>::SudoMsg,
        ) -> Result<::cosmwasm_std::Response, <$app_type as $crate::sdk::base::Handler>::Error> {
            use $crate::sdk::base::SudoEndpoint;
            $app_const.sudo(deps, env, msg)
        }

        pub type InstantiateMsg =
            <$app_type as $crate::sdk::base::InstantiateEndpoint>::InstantiateMsg;
        pub type ExecuteMsg = <$app_type as $crate::sdk::base::ExecuteEndpoint>::ExecuteMsg;
        pub type QueryMsg = <$app_type as $crate::sdk::base::QueryEndpoint>::QueryMsg;
        pub type MigrateMsg = <$app_type as $crate::sdk::base::MigrateEndpoint>::MigrateMsg;
    };
}

#[macro_export]
#[doc(hidden)]
macro_rules! __cw_orch_interfaces__ {
    ($interface_name: ident) => {
        #[cfg(not(target_arch = "wasm32"))]
        impl<Chain: ::cw_orch::prelude::CwEnv> ::cw_orch::prelude::Uploadable
            for $interface_name<Chain>
        {
            fn wasm(_chain: &::cw_orch::prelude::ChainInfoOwned) -> ::cw_orch::prelude::WasmPath {
                let wasm_name = env!("CARGO_CRATE_NAME").replace('-', "_");
                ::cw_orch::prelude::ArtifactsDir::auto(Some(env!("CARGO_MANIFEST_DIR").to_string()))
                    .find_wasm_path(&wasm_name)
                    .unwrap()
            }

            fn wrapper() -> Box<
                dyn ::cw_orch::prelude::MockContract<::cosmwasm_std::Empty, ::cosmwasm_std::Empty>,
            > {
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

        #[cfg(not(target_arch = "wasm32"))]
        impl<Chain: ::cw_orch::prelude::CwEnv> $crate::abstract_interface::AppDeployer<Chain>
            for $interface_name<Chain>
        {
        }

        #[cfg(not(target_arch = "wasm32"))]
        impl<T: ::cw_orch::prelude::CwEnv> From<::cw_orch::contract::Contract<T>>
            for $interface_name<T>
        {
            fn from(contract: ::cw_orch::contract::Contract<T>) -> Self {
                Self(contract)
            }
        }
    };
}
