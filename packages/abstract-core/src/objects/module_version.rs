/*!
Most of the CW* specs are focused on the *public interfaces*
of the module. The Adapters used for `ExecuteMsg` or `QueryMsg`.
However, when we wish to migrate or inspect smart module info,
we need some form of smart module information embedded on state.

This is where ModuleData comes in. It specifies a special Item to
be stored on disk by all contracts on `instantiate`.

`ModuleInfo` must be stored under the `"module_info"` key which translates
to `"636F6E74726163745F696E666F"` in hex format.
Since the state is well-defined, we do not need to support any "smart queries".
We do provide a helper to construct a "raw query" to read the ContractInfo
of any CW2-compliant module.

For more information on this specification, please check out the
[README](https://github.com/CosmWasm/cw-plus/blob/main/packages/cw2/README.md).
 */

use super::dependency::{Dependency, StaticDependency};
use crate::AbstractError;
use cosmwasm_std::{
    ensure, ensure_eq, Empty, Querier, QuerierWrapper, QueryRequest, StdResult, Storage, WasmQuery,
};
use cw2::{get_contract_version, ContractVersion};
use cw_storage_plus::Item;
use semver::Version;
use serde::{Deserialize, Serialize};

// ANCHOR: metadata
pub const MODULE: Item<ModuleData> = Item::new("module_data");

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct ModuleData {
    /// The name of the module, which should be composed of
    /// the publisher's namespace and module id. eg. `cw-plus:cw20-base`
    pub module: String,
    /// Semantic version of the module's crate on release.
    /// Is used for migration assertions
    pub version: String,
    /// List of modules that this module depends on
    /// along with its version requirements.
    pub dependencies: Vec<Dependency>,
    /// URL to data that follows the Abstract metadata standard for
    /// resolving off-chain module information.
    pub metadata: Option<String>,
}
// ANCHOR_END: metadata

/// get_module_version can be use in migrate to read the previous version of this module
pub fn get_module_data(store: &dyn Storage) -> StdResult<ModuleData> {
    MODULE.load(store).map_err(Into::into)
}

/// set_module_version should be used in instantiate to store the original version, and after a successful
/// migrate to update it
pub fn set_module_data<T: Into<String>, U: Into<String>, M: Into<String>>(
    store: &mut dyn Storage,
    name: T,
    version: U,
    dependencies: &[StaticDependency],
    metadata: Option<M>,
) -> StdResult<()> {
    let val = ModuleData {
        module: name.into(),
        version: version.into(),
        dependencies: dependencies.iter().map(Into::into).collect(),
        metadata: metadata.map(Into::into),
    };
    MODULE.save(store, &val).map_err(Into::into)
}

/// Assert that the new version is greater than the stored version.
pub fn assert_contract_upgrade(
    storage: &dyn Storage,
    to_contract: impl ToString,
    to_version: Version,
) -> Result<(), AbstractError> {
    let ContractVersion {
        version: from_version,
        contract,
    } = get_contract_version(storage)?;

    let to_contract = to_contract.to_string();

    // Must be the same contract
    ensure_eq!(
        contract,
        to_contract,
        AbstractError::ContractNameMismatch {
            from: contract,
            to: to_contract,
        }
    );

    let from_version = from_version.parse().unwrap();

    // Must be a version upgrade
    ensure!(
        to_version > from_version,
        AbstractError::CannotDowngradeContract {
            contract,
            from: from_version,
            to: to_version,
        }
    );
    Ok(())
}

/// Assert that the new version is greater than the stored version.
pub fn assert_cw_contract_upgrade(
    storage: &dyn Storage,
    to_contract: impl ToString,
    to_version: cw_semver::Version,
) -> Result<(), AbstractError> {
    assert_contract_upgrade(
        storage,
        to_contract,
        to_version.to_string().parse().unwrap(),
    )
}

/// Migrate the module data to the new state.
/// If there was no moduleData stored, it will be set to the given values with an empty dependency array.
/// If the metadata is `None`, the old metadata will be kept.
/// If the metadata is `Some`, the old metadata will be overwritten.
pub fn migrate_module_data(
    store: &mut dyn Storage,
    name: &str,
    version: &str,
    metadata: Option<String>,
) -> StdResult<()> {
    let old_module_data = MODULE.may_load(store)?;
    let val = old_module_data.map_or(
        ModuleData {
            module: name.into(),
            version: version.into(),
            dependencies: vec![],
            metadata: None,
        },
        |data| ModuleData {
            module: name.into(),
            version: version.into(),
            dependencies: data.dependencies,
            metadata: metadata.or(data.metadata),
        },
    );

    MODULE.save(store, &val).map_err(Into::into)
}

/// This will make a raw_query to another module to determine the current version it
/// claims to be. This should not be trusted, but could be used as a quick filter
/// if the other module exists and claims to be a cw20-base module for example.
/// (Note: you usually want to require *interfaces* not *implementations* of the
/// contracts you compose with, so be careful of overuse)
pub fn query_module_data<Q: Querier, T: Into<String>>(
    querier: &Q,
    contract_addr: T,
) -> StdResult<ModuleData> {
    let req = QueryRequest::Wasm(WasmQuery::Raw {
        contract_addr: contract_addr.into(),
        key: MODULE.as_slice().into(),
    });
    QuerierWrapper::<Empty>::new(querier)
        .query(&req)
        .map_err(Into::into)
}

#[cfg(test)]
mod tests {
    use super::*;
    use cosmwasm_std::testing::MockStorage;

    #[test]
    fn get_and_set_work() {
        let mut store = MockStorage::new();

        // error if not set
        assert!(get_module_data(&store).is_err());

        // set and get
        let contract_name = "crate:cw20-base";
        let contract_version = "0.2.0";
        let metadata = Some("https://example.com");
        const REQUIREMENT: [&str; 1] = [">1"];

        const DEPENDENCIES: &[StaticDependency; 1] = &[StaticDependency {
            id: "abstact::dex",
            version_req: &REQUIREMENT,
        }];
        set_module_data(
            &mut store,
            contract_name,
            contract_version,
            DEPENDENCIES,
            metadata,
        )
        .unwrap();

        let loaded = get_module_data(&store).unwrap();
        let expected = ModuleData {
            module: contract_name.to_string(),
            version: contract_version.to_string(),
            dependencies: DEPENDENCIES.iter().map(Into::into).collect(),
            metadata: metadata.map(Into::into),
        };
        assert_eq!(expected, loaded);
    }
}
