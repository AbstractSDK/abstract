use crate::{state::ContractError, AppContract, Handler, MigrateEndpoint};
use abstract_core::objects::module_version::assert_contract_upgrade;
use abstract_core::{app::MigrateMsg, objects::module_version::set_module_data};
use cosmwasm_std::Response;
use cw2::set_contract_version;
use schemars::JsonSchema;
use serde::Serialize;

impl<
        Error: ContractError,
        CustomInitMsg,
        CustomExecMsg,
        CustomQueryMsg,
        CustomMigrateMsg: Serialize + JsonSchema,
        ReceiveMsg,
        SudoMsg,
    > MigrateEndpoint
    for AppContract<
        Error,
        CustomInitMsg,
        CustomExecMsg,
        CustomQueryMsg,
        CustomMigrateMsg,
        ReceiveMsg,
        SudoMsg,
    >
{
    type MigrateMsg = MigrateMsg<CustomMigrateMsg>;

    fn migrate(
        self,
        deps: cosmwasm_std::DepsMut,
        env: cosmwasm_std::Env,
        msg: Self::MigrateMsg,
    ) -> Result<cosmwasm_std::Response, Self::Error> {
        let (name, version_string, metadata) = self.info();
        let to_version = version_string.parse().unwrap();
        assert_contract_upgrade(deps.storage, name, to_version)?;
        set_module_data(
            deps.storage,
            name,
            version_string,
            self.dependencies(),
            metadata,
        )?;
        set_contract_version(deps.storage, name, version_string)?;
        if let Some(migrate_fn) = self.maybe_migrate_handler() {
            return migrate_fn(deps, env, self, msg.module);
        }
        Ok(Response::default())
    }
}
