use crate::{Handler, MigrateEndpoint};
use abstract_sdk::os::app::MigrateMsg;
use cosmwasm_std::{Response, StdError};
use cw2::{get_contract_version, set_contract_version};
use schemars::JsonSchema;
use semver::Version;
use serde::Serialize;

use crate::{AppContract, AppError};

impl<
        Error: From<cosmwasm_std::StdError> + From<AppError>,
        CustomExecMsg,
        CustomInitMsg,
        CustomQueryMsg,
        CustomMigrateMsg: Serialize + JsonSchema,
        ReceiveMsg,
    > MigrateEndpoint
    for AppContract<
        Error,
        CustomExecMsg,
        CustomInitMsg,
        CustomQueryMsg,
        CustomMigrateMsg,
        ReceiveMsg,
    >
{
    type MigrateMsg = MigrateMsg<CustomMigrateMsg>;

    fn migrate(
        self,
        deps: cosmwasm_std::DepsMut,
        env: cosmwasm_std::Env,
        msg: Self::MigrateMsg,
    ) -> Result<cosmwasm_std::Response, Self::Error> {
        let (name, version_string) = self.info();
        let version: Version =
            Version::parse(version_string).map_err(|e| StdError::generic_err(e.to_string()))?;
        let storage_version: Version = get_contract_version(deps.storage)?.version.parse().unwrap();
        if storage_version < version {
            set_contract_version(deps.storage, name, version_string)?;
        }
        if let Some(migrate_fn) = self.maybe_migrate_handler() {
            return migrate_fn(deps, env, self, msg.app);
        }
        Ok(Response::default())
    }
}
