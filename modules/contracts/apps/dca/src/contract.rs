use abstract_app::AppContract;
use abstract_app::abstract_core::objects::dependency::StaticDependency;
#[cfg(feature = "interface")]
use abstract_app::abstract_core::{manager::ModuleInstallConfig, objects::module::ModuleInfo};
use cosmwasm_std::{Empty, Response};
#[cfg(feature = "interface")]
use croncat_app::contract::interface::Croncat;
use croncat_app::contract::{CRONCAT_ID, CRONCAT_MODULE_VERSION};

use crate::{
    error::DCAError,
    handlers,
    msg::{AppInstantiateMsg, DCAExecuteMsg, DCAQueryMsg},
};

/// The version of your app
pub const DCA_APP_VERSION: &str = env!("CARGO_PKG_VERSION");
/// The id of the app
pub const DCA_APP_ID: &str = "abstract:dca";

/// The type of the result returned by your app's entry points.
pub type AppResult<T = Response> = Result<T, DCAError>;

/// The type of the app that is used to build your app and access the Abstract SDK features.
pub type DCAApp = AppContract<DCAError, AppInstantiateMsg, DCAExecuteMsg, DCAQueryMsg, Empty>;

const DCA_APP: DCAApp = DCAApp::new(DCA_APP_ID, DCA_APP_VERSION, None)
    .with_instantiate(handlers::instantiate_handler)
    .with_execute(handlers::execute_handler)
    .with_query(handlers::query_handler)
    .with_dependencies(&[
        StaticDependency::new(CRONCAT_ID, &[CRONCAT_MODULE_VERSION]),
        StaticDependency::new(
            abstract_dex_adapter::DEX_ADAPTER_ID,
            &[abstract_dex_adapter::contract::CONTRACT_VERSION],
        ),
    ]);

// Export handlers
#[cfg(feature = "export")]
abstract_app::export_endpoints!(DCA_APP, DCAApp);

#[cfg(feature = "interface")]
abstract_app::cw_orch_interface!(DCA_APP, DCAApp, DCA);

#[cfg(feature = "interface")]
impl<Chain: cw_orch::environment::CwEnv> abstract_app::abstract_interface::DependencyCreation
    for crate::DCA<Chain>
{
    type DependenciesConfig = cosmwasm_std::Empty;

    fn dependency_install_configs(
        _configuration: Self::DependenciesConfig,
    ) -> Result<Vec<ModuleInstallConfig>, abstract_app::abstract_interface::AbstractInterfaceError> {
        let croncat_dependency_install_configs: Vec<ModuleInstallConfig> =
            <Croncat<Chain> as abstract_app::abstract_interface::DependencyCreation>::dependency_install_configs(
                cosmwasm_std::Empty {},
            )?;
        let adapter_install_config = ModuleInstallConfig::new(
            ModuleInfo::from_id(
                abstract_dex_adapter::DEX_ADAPTER_ID,
                abstract_dex_adapter::contract::CONTRACT_VERSION.into(),
            )?,
            None,
        );
        let croncat_install_config =
            <Croncat<Chain> as abstract_app::abstract_interface::InstallConfig>::install_config(
                &croncat_app::msg::AppInstantiateMsg {},
            )?;

        Ok([
            croncat_dependency_install_configs,
            vec![croncat_install_config, adapter_install_config],
        ]
        .concat())
    }
}
