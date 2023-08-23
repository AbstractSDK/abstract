pub enum AdapterOptions {
    Deploy,
    // TODO add other useful options for adapters
    // and ability to add customs as wel
}

impl std::fmt::Display for AdapterOptions {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            AdapterOptions::Deploy => f.pad("Deploy"),
        }
    }
}

#[derive(Clone)]
pub struct AdapterContext<CustomInitMsg: serde::Serialize> {
    pub version: semver::Version,
    pub init_msg: CustomInitMsg
}

#[macro_export]
macro_rules! cw_cli {
    ($adapter_type:ident, $init_msg: ty) => {
        mod implement_addons {
            impl ::cw_orch_cli::CwCliAddons<::abstract_adapter::cli::AdapterContext<$init_msg>>
                for super::interface::$adapter_type<::cw_orch::daemon::Daemon>
            {
                fn addons(
                    &mut self,
                    context: ::abstract_adapter::cli::AdapterContext<$init_msg>,
                ) -> ::cw_orch_cli::OrchCliResult<()>
                where
                    Self: ::cw_orch::prelude::ContractInstance<::cw_orch::daemon::Daemon>,
                {
                    let option =
                        ::cw_orch_cli::select_msg(vec![::abstract_adapter::cli::AdapterOptions::Deploy])?;
                    match option {
                        ::abstract_adapter::cli::AdapterOptions::Deploy => {
                            ::abstract_interface::AdapterDeployer::deploy(self, context.version, context.init_msg)
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
