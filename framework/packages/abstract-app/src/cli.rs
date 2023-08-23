pub enum AppOptions {
    Deploy,
    // TODO add other useful options for apps
    // and ability to add customs as wel
}

impl std::fmt::Display for AppOptions {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            AppOptions::Deploy => f.pad("Deploy"),
        }
    }
}

#[derive(Clone)]
pub struct AppContext {
    pub version: semver::Version,
}

#[macro_export]
macro_rules! cw_cli {
    ($app_type:ident) => {
        mod implement_addons {
            impl ::cw_orch_cli::CwCliAddons<::abstract_app::cli::AppContext>
                for super::interface::$app_type<::cw_orch::daemon::Daemon>
            {
                fn addons(
                    &mut self,
                    context: ::abstract_app::cli::AppContext,
                ) -> ::cw_orch_cli::OrchCliResult<()>
                where
                    Self: ::cw_orch::prelude::ContractInstance<::cw_orch::daemon::Daemon>,
                {
                    let option =
                        ::cw_orch_cli::select_msg(vec![::abstract_app::cli::AppOptions::Deploy])?;
                    match option {
                        ::abstract_app::cli::AppOptions::Deploy => {
                            ::abstract_interface::AppDeployer::deploy(self, context.version)
                                .map_err(|e| ::cw_orch_cli::OrchCliError::CustomError {
                                    val: e.to_string(),
                                })
                        }
                    }
                }
            }
        }
    };
}
