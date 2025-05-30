use abstract_sdk::std::ACCOUNT;
use abstract_std::account::MigrateMsg;
use abstract_std::objects::module_version::assert_contract_upgrade;

use abstract_std::AbstractError;
use cosmwasm_std::{DepsMut, Env};
use cw2::{get_contract_version, set_contract_version};
use semver::Version;

use crate::contract::{AccountResponse, AccountResult, CONTRACT_VERSION};

/// This migration function allows migrating from 2 types of contract
/// - Previous Abstract Account version (This is the first part of the function)
/// - XION Account, to allow upgrading their account to a more feature rich account (second part of the function)
/// All other contracts cannot be migrated to this version
#[cfg_attr(feature = "export", cosmwasm_std::entry_point)]
pub fn migrate(deps: DepsMut, _env: Env, _msg: MigrateMsg) -> AccountResult {
    let version: Version = CONTRACT_VERSION.parse().unwrap();

    let current_contract_version = get_contract_version(deps.storage)?;
    // If we already have an abstract account, we just migrate like normal
    if current_contract_version.contract == ACCOUNT {
        assert_contract_upgrade(deps.storage, ACCOUNT, version)?;
        set_contract_version(deps.storage, ACCOUNT, CONTRACT_VERSION)?;
        Ok(AccountResponse::action("migrate"))
    } else {
        #[cfg(feature = "xion")]
        {
            if current_contract_version.contract != "account" {
                Err(AbstractError::ContractNameMismatch {
                    from: current_contract_version.contract.clone(),
                    to: ACCOUNT.to_string(),
                })?;
            }
            // We might want to migrate from a XION account
            migrate_from_xion_account(
                deps,
                _env,
                current_contract_version,
                _msg.code_id
                    .ok_or(crate::error::AccountError::MissingCodeIdToMigrate {})?,
            )
        }
        #[cfg(not(feature = "xion"))]
        {
            Err(AbstractError::ContractNameMismatch {
                from: current_contract_version.contract,
                to: ACCOUNT.to_string(),
            })?
        }
    }
}

#[cfg(feature = "xion")]
pub fn migrate_from_xion_account(
    mut deps: DepsMut,
    env: Env,
    current_contract_version: cw2::ContractVersion,
    new_code_id: u64,
) -> AccountResult {
    use crate::modules::{_install_modules, MIGRATE_CONTEXT};
    use ::{
        abstract_sdk::feature_objects::RegistryContract,
        abstract_sdk::std::account::state::ACCOUNT_ID,
        abstract_std::account::ModuleInstallConfig,
        abstract_std::objects::module::ModuleInfo,
        abstract_std::objects::AccountId,
        abstract_std::IBC_CLIENT,
        abstract_std::{
            account::state::{WhitelistedModules, SUSPENSION_STATUS, WHITELISTED_MODULES},
            objects::{
                gov_type::GovernanceDetails,
                ownership::{self},
            },
            registry::state::LOCAL_ACCOUNT_SEQUENCE,
        },
        cosmwasm_std::wasm_execute,
    };

    if current_contract_version.contract != "account" {
        Err(AbstractError::ContractNameMismatch {
            from: current_contract_version.contract,
            to: ACCOUNT.to_string(),
        })?;
    }
    // Use CW2 to set the contract version, this is needed for migrations
    set_contract_version(deps.storage, ACCOUNT, CONTRACT_VERSION)?;

    let registry = RegistryContract::new(deps.as_ref(), new_code_id)?;

    let account_id =
        AccountId::local(LOCAL_ACCOUNT_SEQUENCE.query(&deps.querier, registry.address.clone())?);

    let mut response = AccountResponse::new(
        "migrate",
        vec![("account_id".to_owned(), account_id.to_string())],
    );

    ACCOUNT_ID.save(deps.storage, &account_id)?;
    WHITELISTED_MODULES.save(deps.storage, &WhitelistedModules(vec![]))?;

    MIGRATE_CONTEXT.save(deps.storage, &vec![])?;

    let governance = GovernanceDetails::AbstractAccount {
        address: env.contract.address.clone(),
    };

    // Set owner
    let cw_gov_owner = ownership::initialize_owner(deps.branch(), governance)?;

    SUSPENSION_STATUS.save(deps.storage, &false)?;

    response = response.add_attribute("owner".to_owned(), cw_gov_owner.owner.to_string());

    response = response.add_message(wasm_execute(
        registry.address,
        &abstract_std::registry::ExecuteMsg::AddAccount {
            namespace: None,
            creator: env.contract.address.to_string(),
        },
        vec![],
    )?);

    let install_modules = vec![ModuleInstallConfig::new(
        ModuleInfo::from_id_latest(IBC_CLIENT)?,
        None,
    )];

    // Install IBC Client module
    let (install_msgs, install_attribute) =
        _install_modules(deps, install_modules, vec![], new_code_id)?;
    response = response
        .add_submessages(install_msgs)
        .add_attribute(install_attribute.key, install_attribute.value);

    Ok(response)
}

#[cfg(test)]
mod tests {
    use abstract_testing::{abstract_mock_querier, mock_env_validated};
    use cosmwasm_std::testing::*;
    use semver::Version;

    use super::*;
    use crate::error::AccountError;
    use crate::test_common::mock_init;

    use abstract_std::{account::MigrateMsg, AbstractError};
    use cw2::get_contract_version;

    #[coverage_helper::test]
    fn disallow_same_version() -> AccountResult<()> {
        let mut deps = mock_dependencies();
        deps.querier = abstract_mock_querier(deps.api);
        let env = mock_env_validated(deps.api);
        mock_init(&mut deps)?;

        let version: Version = CONTRACT_VERSION.parse().unwrap();

        let res = super::migrate(deps.as_mut(), env, MigrateMsg { code_id: None });

        assert_eq!(
            res,
            Err(AccountError::Abstract(
                AbstractError::CannotDowngradeContract {
                    contract: ACCOUNT.to_string(),
                    from: version.clone(),
                    to: version,
                },
            ))
        );

        Ok(())
    }

    #[coverage_helper::test]
    fn disallow_downgrade() -> AccountResult<()> {
        let mut deps = mock_dependencies();
        deps.querier = abstract_mock_querier(deps.api);
        let env = mock_env_validated(deps.api);
        mock_init(&mut deps)?;

        let big_version = "999.999.999";
        set_contract_version(deps.as_mut().storage, ACCOUNT, big_version)?;

        let version: Version = CONTRACT_VERSION.parse().unwrap();

        let res = super::migrate(deps.as_mut(), env, MigrateMsg { code_id: None });

        assert_eq!(
            res,
            Err(AccountError::Abstract(
                AbstractError::CannotDowngradeContract {
                    contract: ACCOUNT.to_string(),
                    from: big_version.parse().unwrap(),
                    to: version,
                },
            ))
        );

        Ok(())
    }

    #[coverage_helper::test]
    fn disallow_name_change() -> AccountResult<()> {
        let mut deps = mock_dependencies();
        deps.querier = abstract_mock_querier(deps.api);
        let env = mock_env_validated(deps.api);
        mock_init(&mut deps)?;

        let old_version = "0.0.0";
        let old_name = "old:contract";
        set_contract_version(deps.as_mut().storage, old_name, old_version)?;

        let res = super::migrate(deps.as_mut(), env, MigrateMsg { code_id: None });

        assert_eq!(
            res,
            Err(AccountError::Abstract(
                AbstractError::ContractNameMismatch {
                    from: old_name.parse().unwrap(),
                    to: ACCOUNT.parse().unwrap(),
                },
            ))
        );

        Ok(())
    }

    #[coverage_helper::test]
    fn works() -> AccountResult<()> {
        let mut deps = mock_dependencies();
        deps.querier = abstract_mock_querier(deps.api);
        let env = mock_env_validated(deps.api);
        mock_init(&mut deps)?;

        let version: Version = CONTRACT_VERSION.parse().unwrap();

        let small_version = Version {
            minor: version.minor - 1,
            ..version.clone()
        }
        .to_string();

        set_contract_version(deps.as_mut().storage, ACCOUNT, small_version)?;

        let res = super::migrate(deps.as_mut(), env, MigrateMsg { code_id: None })?;
        assert!(res.messages.is_empty());

        assert_eq!(
            get_contract_version(&deps.storage)?.version,
            version.to_string()
        );
        Ok(())
    }
}
