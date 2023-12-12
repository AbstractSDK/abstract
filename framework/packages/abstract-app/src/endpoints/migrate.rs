use crate::{state::ContractError, AppContract, Handler, MigrateEndpoint};
use abstract_core::objects::module_version::assert_contract_upgrade;
use abstract_core::{app::MigrateMsg, objects::module_version::set_module_data};
use abstract_sdk::features::{DepsAccess, ResponseGenerator};
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
        '_,
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

    fn migrate(mut self, msg: Self::MigrateMsg) -> Result<cosmwasm_std::Response, Self::Error> {
        let (name, version_string, metadata) = self.info();
        let dependencies = self.dependencies();
        let to_version = version_string.parse().unwrap();
        assert_contract_upgrade(self.deps().storage, name.clone(), to_version)?;
        set_module_data(
            self.deps_mut().storage,
            name.clone(),
            version_string.clone(),
            dependencies,
            metadata,
        )?;
        set_contract_version(self.deps_mut().storage, &name, &version_string)?;
        if let Some(migrate_fn) = self.maybe_migrate_handler() {
            migrate_fn(&mut self, msg.module)?;
            return Ok(self._generate_response()?);
        }
        Ok(Response::default())
    }
}
