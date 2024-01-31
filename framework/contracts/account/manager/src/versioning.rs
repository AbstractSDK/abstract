use abstract_core::{
    manager::state::{ACCOUNT_MODULES, DEPENDENTS},
    objects::{dependency::Dependency, module_version::MODULE},
};
use cosmwasm_std::{Deps, DepsMut, StdError, Storage};
use cw_semver::{Comparator, Version};

use crate::{commands::MIGRATE_CONTEXT, contract::ManagerResult, error::ManagerError};

/// Assert the dependencies that this app relies on are installed.
pub fn assert_install_requirements(deps: Deps, module_id: &str) -> ManagerResult<Vec<Dependency>> {
    let module_dependencies = load_module_dependencies(deps, module_id)?;
    assert_dependency_requirements(deps, &module_dependencies, module_id)?;
    Ok(module_dependencies)
}

/// Assert that the new version of this app is supported by its dependents.
/// Unless that dependent is also being migrated.
pub fn assert_migrate_requirements(
    deps: Deps,
    module_id: &str,
    new_version: Version,
) -> ManagerResult<()> {
    // load all the modules that depend on this module
    let dependents = DEPENDENTS
        .may_load(deps.storage, module_id)?
        .unwrap_or_default();
    let migrating = MIGRATE_CONTEXT.load(deps.storage)?;
    // for each module that depends on this module, check if it supports the new version.
    for dependent_module in dependents {
        // if the dependent is also being migrated, skip it.
        if migrating.iter().any(|(m, _)| m == &dependent_module) {
            continue;
        }
        let dependent_address = ACCOUNT_MODULES.load(deps.storage, &dependent_module)?;
        let module_data = MODULE.query(&deps.querier, dependent_address)?;
        // filter the dependencies and assert version comparison when applicable
        let mut applicable_bounds = module_data
            .dependencies
            .iter()
            .filter(|dep| dep.id == module_id);
        // assert bounds
        applicable_bounds.try_for_each(|dep| {
            assert_comparators(&dep.version_req, &new_version, module_id, false)
        })?;
    }
    Ok(())
}

/// Add module as dependent on its dependencies.
/// For example, Autocompounder depends on dex.
/// Therefore, autocompounder is added as a dependent on dex.
/// dex -> Autocompounder
pub fn set_as_dependent(
    store: &mut dyn Storage,
    module_id: String,
    dependencies: Vec<Dependency>,
) -> ManagerResult<()> {
    for dep in dependencies {
        DEPENDENTS.update(store, &dep.id, |dependents| {
            let mut dependents = dependents.unwrap_or_default();
            dependents.insert(module_id.clone());
            Ok::<_, StdError>(dependents)
        })?;
    }
    Ok(())
}

/// Remove a module as dependent on its dependencies
/// For example, Autocompounder depends on dex.
/// We are uninstalling autocompounder, so we remove it from the dependents of dex.
pub fn remove_as_dependent(
    store: &mut dyn Storage,
    module_id: &str,
    dependencies: Vec<Dependency>,
) -> ManagerResult<()> {
    for dep in dependencies {
        DEPENDENTS.update(store, &dep.id, |dependents| {
            let mut dependents = dependents.unwrap_or_default();
            dependents.remove(module_id);
            Ok::<_, StdError>(dependents)
        })?;
    }
    Ok(())
}

fn assert_comparators(
    bounds: &[Comparator],
    version: &Version,
    module_id: &str,
    post_migration: bool,
) -> ManagerResult<()> {
    // assert requirements
    bounds.iter().try_for_each(|comp: &Comparator| {
        if comp.matches(version) {
            Ok(())
        } else {
            Err(ManagerError::VersionRequirementNotMet {
                module_id: module_id.to_string(),
                version: version.to_string(),
                comp: comp.to_string(),
                post_migration,
            })
        }
    })?;
    Ok(())
}

/// Goes over all the provided dependencies and asserts that:
/// 1. The dependency is installed
/// 2. The dependency version fits the requirements
pub fn assert_dependency_requirements(
    deps: Deps,
    dependencies: &[Dependency],
    dependent: &str,
) -> ManagerResult<()> {
    for dep in dependencies {
        let dep_addr = ACCOUNT_MODULES
            .may_load(deps.storage, &dep.id)?
            .ok_or_else(|| ManagerError::DependencyNotMet(dep.id.clone(), dependent.to_string()))?;

        let dep_version = cw2::CONTRACT.query(&deps.querier, dep_addr)?;
        let version: Version = dep_version.version.parse().unwrap();
        // assert requirements
        assert_comparators(&dep.version_req, &version, &dep.id, true)?;
    }
    Ok(())
}

pub fn load_module_dependencies(deps: Deps, module_id: &str) -> ManagerResult<Vec<Dependency>> {
    let querier = &deps.querier;
    let module_addr = ACCOUNT_MODULES.load(deps.storage, module_id)?;
    let module_data = MODULE.query(querier, module_addr)?;
    Ok(module_data.dependencies)
}

pub fn maybe_remove_old_deps(
    deps: DepsMut,
    module_id: &str,
    old_deps: &[Dependency],
) -> ManagerResult<()> {
    let new_deps = load_module_dependencies(deps.as_ref(), module_id)?;
    // find deps that are no longer required.
    // ie. the old deps contain a deps that the new deps doesn't.
    let removable_deps: Vec<&Dependency> =
        old_deps.iter().filter(|d| !new_deps.contains(d)).collect();
    for dep_to_remove in removable_deps {
        // Remove module from dependents on the removable dep
        DEPENDENTS.update(deps.storage, &dep_to_remove.id, |dependents| {
            // Migrating so hashset should be saved and contain the module ID.
            let mut dependents = dependents.unwrap();
            dependents.remove(module_id);
            Ok::<_, StdError>(dependents)
        })?;
    }
    Ok(())
}

pub fn maybe_add_new_deps(
    deps: DepsMut,
    module_id: &str,
    old_deps: &[Dependency],
) -> ManagerResult<Vec<Dependency>> {
    let new_deps = load_module_dependencies(deps.as_ref(), module_id)?;
    // find deps that are new.
    // the new deps contain deps that were not in the old deps.
    let to_be_added_deps: Vec<&Dependency> =
        new_deps.iter().filter(|d| !old_deps.contains(d)).collect();
    for dep_to_add in &to_be_added_deps {
        // Add module as dependent on the new deps
        // Will also run when a version requirement is changed.
        DEPENDENTS.update(deps.storage, &dep_to_add.id, |dependents| {
            // Adding new dep so might be the first entry, hence default to empty set in that case.
            let mut dependents = dependents.unwrap_or_default();
            dependents.insert(module_id.to_string());
            Ok::<_, StdError>(dependents)
        })?;
    }
    Ok(new_deps)
}

#[cfg(test)]
mod test {
    use std::collections::HashSet;

    use cosmwasm_std::testing::mock_dependencies;
    use speculoos::prelude::*;

    use super::*;

    mod set_as_dependent {
        use super::*;

        // This should add dependency -> [module] to the map
        #[test]
        fn add() {
            let mut deps = mock_dependencies();
            let new_module_id = "module";

            let dependency = "dependency";
            let dependencies = vec![Dependency {
                id: dependency.to_string(),
                // no version requirements
                version_req: vec![],
            }];

            set_as_dependent(&mut deps.storage, new_module_id.to_string(), dependencies).unwrap();

            let dependents = DEPENDENTS.load(&deps.storage, dependency).unwrap();

            assert_that(&dependents).has_length(1);
            assert_that(&dependents).contains(new_module_id.to_string());
        }
    }

    mod remove_as_dependent {
        use super::*;

        fn initialize_dependents(deps: DepsMut, module_id: &str, dependents: Vec<String>) {
            DEPENDENTS
                .save(deps.storage, module_id, &HashSet::from_iter(dependents))
                .unwrap();
        }

        // autocompounder depends on dex
        // dex -> autocompounder
        // to uninstall autocompounder, remove dex
        #[test]
        fn remove() {
            let mut deps = mock_dependencies();

            let dex_adapter = "dex";
            let autocompounder = "autocompounder";

            // dex -> autocompounder
            initialize_dependents(deps.as_mut(), dex_adapter, vec![autocompounder.to_string()]);

            let actual_dex_dependents = DEPENDENTS.load(&deps.storage, dex_adapter).unwrap();
            assert_that(&actual_dex_dependents).has_length(1);
            assert_that(&actual_dex_dependents).contains(autocompounder.to_string());

            // the autocompounder depends on the dex
            let autocompounder_dependencies = vec![Dependency {
                id: dex_adapter.to_string(),
                // no version requirements
                version_req: vec![],
            }];

            let res = remove_as_dependent(
                &mut deps.storage,
                autocompounder,
                autocompounder_dependencies,
            );

            assert_that(&res).is_ok();

            let remaining_dex_dependents = DEPENDENTS.load(&deps.storage, dex_adapter).unwrap();
            assert_that(&remaining_dex_dependents).is_empty();
        }
    }
}
