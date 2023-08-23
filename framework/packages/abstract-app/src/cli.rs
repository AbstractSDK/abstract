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

#[macro_export]
macro_rules! cw_cli {
    ($app_type:ident) => {
        pub mod app_cli {
            #[derive(Clone)]
            pub struct AppContext {
                pub version: semver::Version,
            }

            impl ::cw_orch_cli::CwCliAddons<AppContext>
                for super::$app_type<::cw_orch::daemon::Daemon>
            {
                fn addons(&mut self, context: AppContext) -> ::cw_orch_cli::OrchCliResult<()>
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
