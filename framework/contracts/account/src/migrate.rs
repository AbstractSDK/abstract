use abstract_std::{
    account::MigrateMsg, objects::module_version::assert_contract_upgrade, ACCOUNT,
};
use cosmwasm_std::{DepsMut, Env};
use cw2::set_contract_version;
use semver::Version;

use crate::contract::{AccountResponse, AccountResult, CONTRACT_VERSION};

#[cfg_attr(feature = "export", cosmwasm_std::entry_point)]
pub fn migrate(deps: DepsMut, _env: Env, _msg: MigrateMsg) -> AccountResult {
    let version: Version = CONTRACT_VERSION.parse().unwrap();

    assert_contract_upgrade(deps.storage, ACCOUNT, version)?;
    set_contract_version(deps.storage, ACCOUNT, CONTRACT_VERSION)?;
    Ok(AccountResponse::action("migrate"))
}

#[cfg(test)]
mod tests {
    use abstract_testing::mock_env_validated;
    use cosmwasm_std::testing::*;
    use semver::Version;
    use speculoos::prelude::*;

    use super::*;
    use crate::error::AbstractXionError;
    use crate::test_common::mock_init;

    use abstract_std::{account::MigrateMsg, AbstractError};
    use cw2::get_contract_version;

    #[coverage_helper::test]
    fn disallow_same_version() -> AccountResult<()> {
        let mut deps = mock_dependencies();
        let env = mock_env_validated(deps.api);
        mock_init(&mut deps)?;

        let version: Version = CONTRACT_VERSION.parse().unwrap();

        let res = super::migrate(deps.as_mut(), env, MigrateMsg {});

        assert_that!(res)
            .is_err()
            .is_equal_to(AbstractXionError::Abstract(
                AbstractError::CannotDowngradeContract {
                    contract: ACCOUNT.to_string(),
                    from: version.clone(),
                    to: version,
                },
            ));

        Ok(())
    }

    #[coverage_helper::test]
    fn disallow_downgrade() -> AccountResult<()> {
        let mut deps = mock_dependencies();
        let env = mock_env_validated(deps.api);
        mock_init(&mut deps)?;

        let big_version = "999.999.999";
        set_contract_version(deps.as_mut().storage, ACCOUNT, big_version)?;

        let version: Version = CONTRACT_VERSION.parse().unwrap();

        let res = super::migrate(deps.as_mut(), env, MigrateMsg {});

        assert_that!(res)
            .is_err()
            .is_equal_to(AbstractXionError::Abstract(
                AbstractError::CannotDowngradeContract {
                    contract: ACCOUNT.to_string(),
                    from: big_version.parse().unwrap(),
                    to: version,
                },
            ));

        Ok(())
    }

    #[coverage_helper::test]
    fn disallow_name_change() -> AccountResult<()> {
        let mut deps = mock_dependencies();
        let env = mock_env_validated(deps.api);
        mock_init(&mut deps)?;

        let old_version = "0.0.0";
        let old_name = "old:contract";
        set_contract_version(deps.as_mut().storage, old_name, old_version)?;

        let res = super::migrate(deps.as_mut(), env, MigrateMsg {});

        assert_that!(res)
            .is_err()
            .is_equal_to(AbstractXionError::Abstract(
                AbstractError::ContractNameMismatch {
                    from: old_name.parse().unwrap(),
                    to: ACCOUNT.parse().unwrap(),
                },
            ));

        Ok(())
    }

    #[coverage_helper::test]
    fn works() -> AccountResult<()> {
        let mut deps = mock_dependencies();
        let env = mock_env_validated(deps.api);
        mock_init(&mut deps)?;

        let version: Version = CONTRACT_VERSION.parse().unwrap();

        let small_version = Version {
            minor: version.minor - 1,
            ..version.clone()
        }
        .to_string();

        set_contract_version(deps.as_mut().storage, ACCOUNT, small_version)?;

        let res = super::migrate(deps.as_mut(), env, MigrateMsg {})?;
        assert_that!(res.messages).has_length(0);

        assert_that!(get_contract_version(&deps.storage)?.version).is_equal_to(version.to_string());
        Ok(())
    }
}
