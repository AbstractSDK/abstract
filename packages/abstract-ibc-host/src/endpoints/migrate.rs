use abstract_sdk::{
    base::{Handler, MigrateEndpoint},
    feature_objects::AnsHost,
    namespaces::BASE_STATE,
    os::ibc_host::MigrateMsg,
};
use cosmwasm_std::{Response, StdError};
use cw2::{get_contract_version, set_contract_version};
use cw_storage_plus::Item;
use schemars::JsonSchema;
use semver::Version;
use serde::Serialize;

use crate::{state::HostState, Host, HostError};

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
        mut deps: cosmwasm_std::DepsMut,
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

        // Type migration
        let item = Item::new(BASE_STATE);
        let config: old_self::state::HostState = item.load(deps.storage)?;
        let new_state = HostState {
            chain: config.chain,
            cw1_code_id: config.cw1_code_id,
            ans_host: AnsHost {
                address: config.ans_host.address,
            },
        };
        self.base_state.save(deps.storage, &new_state)?;
        self.admin.set(deps.branch(), Some(config.admin))?;
        if let Some(migrate_fn) = self.maybe_migrate_handler() {
            return migrate_fn(deps, env, self, msg.app);
        }
        Ok(Response::default())
    }
}
