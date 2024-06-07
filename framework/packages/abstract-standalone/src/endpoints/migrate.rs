use abstract_std::{
    objects::module_version::{assert_contract_upgrade, set_module_data},
    AbstractError, AbstractResult,
};
use cw2::set_contract_version;

use crate::StandaloneContract;

impl StandaloneContract {
    /// Call this on migration of Standalone
    pub fn migrate(&self, deps: cosmwasm_std::DepsMut) -> AbstractResult<()> {
        let base_state = self.base_state.load(deps.storage)?;
        if !base_state.is_migratable {
            return Err(AbstractError::NotMigratable {
                contract: self.info.0.to_owned(),
            });
        }

        let (name, version_string, metadata) = self.info;
        let to_version = version_string.parse().unwrap();

        assert_contract_upgrade(deps.storage, name, to_version)?;
        set_module_data(
            deps.storage,
            name,
            version_string,
            self.dependencies,
            metadata,
        )?;
        set_contract_version(deps.storage, name, version_string)?;

        Ok(())
    }
}
