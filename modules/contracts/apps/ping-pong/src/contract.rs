use abstract_app::{objects::dependency::StaticDependency, std::IBC_CLIENT, AppContract};
use cosmwasm_std::Response;

use crate::{
    error::AppError,
    handlers, ibc,
    msg::{AppExecuteMsg, AppInstantiateMsg, AppMigrateMsg, AppQueryMsg},
};

/// The version of your app
pub const APP_VERSION: &str = env!("CARGO_PKG_VERSION");
/// The id of the app
pub const APP_ID: &str = "abstract:ping-pong";

/// The type of the result returned by your app's entry points.
pub type AppResult<T = Response> = Result<T, AppError>;

/// The type of the app that is used to build your app and access the Abstract SDK features.
pub type App = AppContract<AppError, AppInstantiateMsg, AppExecuteMsg, AppQueryMsg, AppMigrateMsg>;

const APP: App = App::new(APP_ID, APP_VERSION, None)
    .with_instantiate(handlers::instantiate_handler)
    .with_execute(handlers::execute_handler)
    .with_query(handlers::query_handler)
    .with_dependencies(&[StaticDependency::new(
        IBC_CLIENT,
        &[abstract_ibc_client::contract::CONTRACT_VERSION],
    )])
    .with_module_ibc(ibc::receive_module_ibc)
    .with_ibc_callback(ibc::ibc_callback);

// Export handlers
#[cfg(feature = "export")]
abstract_app::export_endpoints!(APP, App);

abstract_app::cw_orch_interface!(APP, App, AppInterface);

#[cfg(not(target_arch = "wasm32"))]
use abstract_app::std::manager::ModuleInstallConfig;
#[cfg(not(target_arch = "wasm32"))]
impl<Chain: cw_orch::environment::CwEnv> abstract_app::abstract_interface::DependencyCreation
    for crate::AppInterface<Chain>
{
    type DependenciesConfig = cosmwasm_std::Empty;

    fn dependency_install_configs(
        _configuration: Self::DependenciesConfig,
    ) -> Result<Vec<ModuleInstallConfig>, abstract_app::abstract_interface::AbstractInterfaceError>
    {
        Ok(vec![ModuleInstallConfig::new(
            abstract_app::objects::module::ModuleInfo::from_id(
                IBC_CLIENT,
                abstract_ibc_client::contract::CONTRACT_VERSION.into(),
            )?,
            None,
        )])
    }
}
