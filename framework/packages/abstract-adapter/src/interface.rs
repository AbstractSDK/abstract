#[macro_export]
/// Creates the interface for working with the adapter with [cw-orch](https://github.com/AbstractSDK/cw-orchestrator).
/// This generates all the necessary code used to interact with the adapter in an cw-orch environment
///
/// ## Usage
/// The macro takes three arguments:
/// 1. The adapter's constant, declared in `contract.rs`.
/// 2. The adapter's type, declared in `contract.rs`.
/// 3. The name of the interface struct to be generated.
/// ```rust,ignore
/// cw_orch_interface!(ADAPTER, Adapter, AdapterInterface);
/// ```
///
/// This will generate :
/// ```rust,ignore
/// pub mod interface{
///     #[cw_orch::interface(Adapter::InstantiateMsg, Adapter::ExecuteMsg, Adapter::QueryMsg, Adapter::MigrateMsg)]
///     pub struct AdapterInterface;
///
///     impl <Chain: cw_orch::prelude::CwEnv> cw_orch::prelude::Uploadable for AdapterInterface<Chain> {
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
///                     ADAPTER::execute, // This notation, doesn't actually work like so, but we use that to illustrate
///                     ADAPTER::instantiate,
///                     ADAPTER::query,
///                 )
///                 .with_reply(ADAPTER::reply)
///                 .with_migrate(ADAPTER::migrate)
///                 .with_sudo(ADAPTER::sudo),
///             )
///         }
///     }
///     impl<Chain: ::cw_orch::prelude::CwEnv> $crate::abstract_app::abstract_interface::AdapterDeployer<Chain> for AdapterInterface<Chain> {}
/// }
/// ```
macro_rules! cw_orch_interface {
    ($adapter_const:expr, $adapter_type:ty, $init_msg:ty, $interface_name: ident) => {
        #[cfg(not(target_arch = "wasm32"))]
        mod _wrapper_fns {
            use super::*;
            $crate::__wrapper_fns_without_custom__!($adapter_const, $adapter_type);

            pub fn execute(
                deps: ::cosmwasm_std::DepsMut,
                env: ::cosmwasm_std::Env,
                info: ::cosmwasm_std::MessageInfo,
                msg: <$adapter_type as $crate::sdk::base::ExecuteEndpoint>::ExecuteMsg,
            ) -> Result<
                ::cosmwasm_std::Response,
                <$adapter_type as $crate::sdk::base::Handler>::Error,
            > {
                use $crate::sdk::base::ExecuteEndpoint;
                $adapter_const.execute(deps, env, info, msg)
            }
        }

        pub mod interface {
            use super::*;
            #[::cw_orch::interface(
                _wrapper_fns::InstantiateMsg,
                _wrapper_fns::ExecuteMsg,
                _wrapper_fns::QueryMsg,
                ::cosmwasm_std::Empty
            )]
            pub struct $interface_name;

            $crate::__cw_orch_interfaces__!($interface_name);

            #[cfg(not(target_arch = "wasm32"))]
            impl<Chain: ::cw_orch::prelude::CwEnv>
                $crate::abstract_interface::AdapterDeployer<Chain, $init_msg>
                for $interface_name<Chain>
            {
            }

            #[cfg(not(target_arch = "wasm32"))]
            use $crate::sdk::features::ModuleIdentification;
            #[cfg(not(target_arch = "wasm32"))]
            impl<Chain: ::cw_orch::prelude::CwEnv> $crate::abstract_interface::RegisteredModule
                for $interface_name<Chain>
            {
                type InitMsg = ::cosmwasm_std::Empty;

                fn module_id<'a>() -> &'a str {
                    $adapter_const.module_id()
                }

                fn module_version<'a>() -> &'a str {
                    $adapter_const.version()
                }

                fn dependencies<'a>() -> &'a [$crate::std::objects::dependency::StaticDependency] {
                    $crate::sdk::base::Handler::dependencies(&$adapter_const)
                }
            }
        }
    };
    ($adapter_const:expr, $adapter_type:ty, $init_msg:ty, $interface_name: ident, $custom_exec: ty) => {
        #[cfg(not(target_arch = "wasm32"))]
        mod _wrapper_fns {
            use super::*;
            $crate::__wrapper_fns_without_custom__!($adapter_const, $adapter_type);

            pub fn execute(
                deps: ::cosmwasm_std::DepsMut,
                env: ::cosmwasm_std::Env,
                info: ::cosmwasm_std::MessageInfo,
                msg: $custom_exec,
            ) -> Result<
                ::cosmwasm_std::Response,
                <$adapter_type as $crate::sdk::base::Handler>::Error,
            > {
                use $crate::sdk::base::{CustomExecuteHandler, ExecuteEndpoint};
                match CustomExecuteHandler::try_into_base(msg) {
                    Ok(default) => $adapter_const.execute(deps, env, info, default),
                    Err(custom) => custom.custom_execute(deps, env, info, $adapter_const),
                }
            }
        }

        pub mod interface {
            use super::*;
            #[::cw_orch::interface(
                _wrapper_fns::InstantiateMsg,
                $custom_exec,
                _wrapper_fns::QueryMsg,
                ::cosmwasm_std::Empty
            )]
            pub struct $interface_name;

            $crate::__cw_orch_interfaces__!($interface_name);

            #[cfg(not(target_arch = "wasm32"))]
            impl<Chain: ::cw_orch::prelude::CwEnv>
                $crate::abstract_interface::AdapterDeployer<Chain, $init_msg>
                for $interface_name<Chain>
            {
            }

            #[cfg(not(target_arch = "wasm32"))]
            use $crate::sdk::features::ModuleIdentification;
            #[cfg(not(target_arch = "wasm32"))]
            impl<Chain: ::cw_orch::prelude::CwEnv> $crate::abstract_interface::RegisteredModule
                for $interface_name<Chain>
            {
                type InitMsg = ::cosmwasm_std::Empty;

                fn module_id<'a>() -> &'a str {
                    $adapter_const.module_id()
                }

                fn module_version<'a>() -> &'a str {
                    $adapter_const.version()
                }
            }
        }
    };
}

#[macro_export]
#[doc(hidden)]
macro_rules! __wrapper_fns_without_custom__ {
    ($adapter_const: expr, $adapter_type: ty) => {
        pub fn instantiate(
            deps: ::cosmwasm_std::DepsMut,
            env: ::cosmwasm_std::Env,
            info: ::cosmwasm_std::MessageInfo,
            msg: <$adapter_type as $crate::sdk::base::InstantiateEndpoint>::InstantiateMsg,
        ) -> Result<::cosmwasm_std::Response, <$adapter_type as $crate::sdk::base::Handler>::Error>
        {
            use $crate::sdk::base::InstantiateEndpoint;
            $adapter_const.instantiate(deps, env, info, msg)
        }

        pub fn query(
            deps: ::cosmwasm_std::Deps,
            env: ::cosmwasm_std::Env,
            msg: <$adapter_type as $crate::sdk::base::QueryEndpoint>::QueryMsg,
        ) -> Result<::cosmwasm_std::Binary, <$adapter_type as $crate::sdk::base::Handler>::Error> {
            use $crate::sdk::base::QueryEndpoint;
            $adapter_const.query(deps, env, msg)
        }

        pub fn reply(
            deps: ::cosmwasm_std::DepsMut,
            env: ::cosmwasm_std::Env,
            msg: ::cosmwasm_std::Reply,
        ) -> Result<::cosmwasm_std::Response, <$adapter_type as $crate::sdk::base::Handler>::Error>
        {
            use $crate::sdk::base::ReplyEndpoint;
            $adapter_const.reply(deps, env, msg)
        }

        pub fn sudo(
            deps: ::cosmwasm_std::DepsMut,
            env: ::cosmwasm_std::Env,
            msg: <$adapter_type as $crate::sdk::base::Handler>::SudoMsg,
        ) -> Result<::cosmwasm_std::Response, <$adapter_type as $crate::sdk::base::Handler>::Error>
        {
            use $crate::sdk::base::SudoEndpoint;
            $adapter_const.sudo(deps, env, msg)
        }

        pub type InstantiateMsg =
            <$adapter_type as $crate::sdk::base::InstantiateEndpoint>::InstantiateMsg;
        pub type ExecuteMsg = <$adapter_type as $crate::sdk::base::ExecuteEndpoint>::ExecuteMsg;
        pub type QueryMsg = <$adapter_type as $crate::sdk::base::QueryEndpoint>::QueryMsg;
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
            fn wasm(_chain: &cw_orch::prelude::ChainInfoOwned) -> ::cw_orch::prelude::WasmPath {
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
                    .with_sudo(_wrapper_fns::sudo),
                )
            }
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
