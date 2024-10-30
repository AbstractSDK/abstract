use abstract_std::{
    objects::module_version::{assert_contract_upgrade, set_module_data},
    AbstractResult,
};
use cosmwasm_std::StdError;
use cw2::set_contract_version;

use crate::StandaloneContract;

impl StandaloneContract {
    /// Call this on migration of Standalone
    pub fn migrate(&self, deps: cosmwasm_std::DepsMut) -> AbstractResult<()> {
        let base_state = self.base_state.load(deps.storage)?;
        if !base_state.is_migratable {
            return Err(StdError::generic_err("Migration is not enabled on contract").into());
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

#[cfg(test)]
mod test {
    use crate::mock::*;
    use abstract_unit_test_utils::prelude::*;

    #[coverage_helper::test]
    fn test_migrate_migratable() {
        let mut deps = mock_init(true);
        let env = mock_env_validated(deps.api);
        let res = migrate(deps.as_mut(), env, MockMigrateMsg {}).unwrap();
        assert!(res.messages.is_empty());
    }
}
