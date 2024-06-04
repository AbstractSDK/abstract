use abstract_std::{
    objects::module_version::{assert_contract_upgrade, set_module_data},
    AbstractResult,
};
use cw2::set_contract_version;

use crate::StandaloneContract;

impl StandaloneContract {
    /// Call this on migration of Standalone
    pub fn migrate(&self, deps: cosmwasm_std::DepsMut) -> AbstractResult<()> {
        let (name, version_string, metadata) = self.info;
        let to_version = version_string.parse().unwrap();
        assert_contract_upgrade(deps.storage, name, to_version)?;
        set_module_data(deps.storage, name, version_string, &[], metadata)?;
        set_contract_version(deps.storage, name, version_string)?;

        Ok(())
    }
}
