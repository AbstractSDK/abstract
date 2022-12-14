use abstract_os::{
    manager::state::{DEPENDENTS, OS_MODULES},
    objects::{dependency::Dependency, module_version::MODULE},
};
use cosmwasm_std::{Deps, DepsMut, StdError, StdResult, Storage};
use semver::{Comparator, Version};

/// Assert the dependencies that this app relies on are installed.
pub fn assert_install_requirements(deps: Deps, module_id: &str) -> StdResult<Vec<Dependency>> {
    let module_dependencies = module_dependencies(deps, module_id)?;
    assert_dependency_requirements(deps, &module_dependencies)?;
    Ok(module_dependencies)
}

/// Assert that the new version of this app is supported by its dependents.
pub fn assert_migrate_requirements(
    deps: Deps,
    module_id: &str,
    new_version: Version,
) -> StdResult<()> {
    // load all the modules that depend on this module
    let dependents = DEPENDENTS
        .may_load(deps.storage, module_id)?
        .unwrap_or_default();
    // for each module that depends on this module, check if it supports the new version.
    for dependent_module in dependents {
        let dependent_address = OS_MODULES.load(deps.storage, &dependent_module)?;
        let module_data = MODULE.query(&deps.querier, dependent_address)?;
        // filter the dependencies and assert version comparison when applicable
        let mut applicable_bounds = module_data
            .dependencies
            .iter()
            .filter(|dep| dep.id == module_id);
        // assert bounds
        applicable_bounds
            .try_for_each(|dep| assert_comparators(&dep.version_req, &new_version, module_id))?;
    }
    Ok(())
}

/// Add module as dependent on its dependencies.
pub fn set_as_dependent(
    store: &mut dyn Storage,
    module_id: String,
    dependencies: Vec<Dependency>,
) -> StdResult<()> {
    for dep in dependencies {
        DEPENDENTS.update(store, &dep.id, |dependents| {
            let mut dependents = dependents.unwrap_or_default();
            dependents.insert(module_id.clone());
            Ok::<_, StdError>(dependents)
        })?;
    }
    Ok(())
}

/// Add module as dependent on its dependencies.
pub fn remove_as_dependent(
    store: &mut dyn Storage,
    module_id: &str,
    dependencies: Vec<Dependency>,
) -> StdResult<()> {
    for dep in dependencies {
        DEPENDENTS.update(store, &dep.id, |dependents| {
            let mut dependents = dependents.unwrap_or_default();
            dependents.remove(module_id);
            Ok::<_, StdError>(dependents)
        })?;
    }
    Ok(())
}

fn assert_comparators(bounds: &[Comparator], version: &Version, module_id: &str) -> StdResult<()> {
    // assert requirements
    bounds.iter().try_for_each(|comp| {
        if comp.matches(version) {
            Ok(())
        } else {
            Err(StdError::generic_err(format!(
                "Module {} with version {} does not fit requirement {}.",
                module_id, version, comp
            )))
        }
    })?;
    Ok(())
}

/// Goes over all the provided dependencies and asserts that:
/// 1. The module is installed
/// 2. The module version fits the requirements
pub fn assert_dependency_requirements(deps: Deps, dependencies: &[Dependency]) -> StdResult<()> {
    for dep in dependencies {
        let maybe_dep_addr = OS_MODULES.load(deps.storage, &dep.id);
        if maybe_dep_addr.is_err() {
            return Err(StdError::generic_err(format!(
                "Module {} not enabled on OS.",
                dep.id
            )));
        };
        let dep_version = cw2::CONTRACT.query(&deps.querier, maybe_dep_addr?)?;
        let version: Version = dep_version.version.parse().unwrap();
        // assert requirements
        assert_comparators(&dep.version_req, &version, &dep.id)?;
    }
    Ok(())
}

pub fn module_dependencies(deps: Deps, module_id: &str) -> StdResult<Vec<Dependency>> {
    let querier = &deps.querier;
    let module_addr = OS_MODULES.load(deps.storage, module_id)?;
    let module_data = MODULE.query(querier, module_addr)?;
    Ok(module_data.dependencies)
}

pub fn maybe_remove_old_deps(
    deps: DepsMut,
    module_id: &str,
    old_deps: &[Dependency],
) -> StdResult<()> {
    let new_deps = module_dependencies(deps.as_ref(), module_id)?;
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
) -> StdResult<Vec<Dependency>> {
    let new_deps = module_dependencies(deps.as_ref(), module_id)?;
    // find deps that are no longer required.
    // ie. the old deps contain a deps that the new deps doesn't.
    let to_be_added_deps: Vec<&Dependency> =
        new_deps.iter().filter(|d| !old_deps.contains(d)).collect();
    for dep_to_add in &to_be_added_deps {
        // Remove module from dependents on the removable dep
        DEPENDENTS.update(deps.storage, &dep_to_add.id, |dependents| {
            // Adding new dep so might be the first entry, hence default to empty set in that case.
            let mut dependents = dependents.unwrap_or_default();
            dependents.insert(module_id.to_string());
            Ok::<_, StdError>(dependents)
        })?;
    }
    Ok(new_deps)
}
