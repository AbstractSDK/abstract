use abstract_app::std::objects::dependency::StaticDependency;
use abstract_app::AppContract;
use cosmwasm_std::{Empty, Response};
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

abstract_app::cw_orch_interface!(DCA_APP, DCAApp, DCA);

#[cfg(not(target_arch = "wasm32"))]
mod deps {
    use abstract_app::{
        abstract_interface::{AbstractInterfaceError, DependencyCreation, InstallConfig},
        std::manager::ModuleInstallConfig,
    };
    use abstract_dex_adapter::interface::DexAdapter;
    use cosmwasm_std::Empty;
    use croncat_app::Croncat;
    use cw_orch::environment::CwEnv;

    use crate::DCA;

    // ANCHOR: deps_creation
    impl<Chain: CwEnv> DependencyCreation for DCA<Chain> {
        // No external dependency data required for installing this app's deps.
        type DependenciesConfig = Empty;

        fn dependency_install_configs(
            _configuration: Self::DependenciesConfig,
        ) -> Result<Vec<ModuleInstallConfig>, AbstractInterfaceError> {
            let mut dependency_configs = vec![];

            // Get any install configs that might be required by CronCat App.
            dependency_configs.extend(Croncat::<Chain>::dependency_install_configs(Empty {})?);

            // Get the CronCat App install config
            dependency_configs.push(Croncat::<Chain>::install_config(
                &croncat_app::msg::AppInstantiateMsg {},
            )?);

            // Create the adapter install config
            dependency_configs.push(DexAdapter::<Chain>::install_config(&Empty {})?);

            Ok(dependency_configs)
        }
    }
    // ANCHOR_END: deps_creation
}
