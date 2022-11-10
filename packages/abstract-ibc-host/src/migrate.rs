use abstract_os::ibc_host::MigrateMsg;
use abstract_sdk::{Handler, MigrateEndpoint};
use cosmwasm_std::{Response, StdError};
use cw2::{get_contract_version, set_contract_version};
use schemars::JsonSchema;
use semver::Version;
use serde::Serialize;

use crate::{Host, HostError};

impl<
        Error: From<cosmwasm_std::StdError> + From<HostError>,
        CustomExecMsg,
        CustomInitMsg,
        CustomQueryMsg,
        CustomMigrateMsg: Serialize + JsonSchema,
        ReceiveMsg,
    > MigrateEndpoint
    for Host<Error, CustomExecMsg, CustomInitMsg, CustomQueryMsg, CustomMigrateMsg, ReceiveMsg>
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
