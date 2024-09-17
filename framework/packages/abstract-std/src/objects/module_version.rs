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

Additionally, it's worth noting that `ModuleData` is utilized by native
abstract contracts.

For more information on this specification, please check out the
[README](https://github.com/CosmWasm/cw-plus/blob/main/packages/cw2/README.md).
 */

use cosmwasm_std::{
    ensure, ensure_eq, Empty, Querier, QuerierWrapper, QueryRequest, StdResult, Storage, WasmQuery,
};
use cw2::{get_contract_version, ContractVersion};
use cw_storage_plus::Item;
use semver::Version;
use serde::{Deserialize, Serialize};

use super::{
    dependency::{Dependency, DependencyResponse, StaticDependency},
    storage_namespaces::MODULE_STORAGE_KEY,
};
use crate::AbstractError;

// ANCHOR: metadata
pub const MODULE: Item<ModuleData> = Item::new(MODULE_STORAGE_KEY);

/// Represents metadata for abstract modules and abstract native contracts.
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

#[cosmwasm_schema::cw_serde]
pub struct ModuleDataResponse {
    pub module_id: String,
    pub version: String,
    pub dependencies: Vec<DependencyResponse>,
    pub metadata: Option<String>,
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
    // Must be 1 major or 1 minor version bump, not more
    // Patches we ignore
    let major_diff = to_version.major.checked_sub(from_version.major);
    let minor_diff = to_version.minor.checked_sub(from_version.minor);
    let no_skips = match (major_diff, minor_diff) {
        // 1) major upgrade - minor should stay the same (1.0.0 -> 2.0.0)
        // 2) major upgrade - minor sub overflowed (0.1.0 -> 1.0.0)
        (Some(1), _) => true,
        // minor upgrade - major should stay the same (1.0.0 -> 1.1.0)
        (Some(0), Some(1)) => true,
        // patch upgrade - minor and major stays the same (1.0.0 -> 1.0.1)
        (Some(0), Some(0)) => true,
        _ => false,
    };
    ensure!(
        no_skips,
        AbstractError::CannotSkipVersion {
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
    to_version: semver::Version,
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
    use cosmwasm_std::testing::MockStorage;

    use super::*;

    #[test]
    fn set_works() {
        let mut store = MockStorage::new();

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

        let loaded = MODULE.load(&store).unwrap();
        let expected = ModuleData {
            module: contract_name.to_string(),
            version: contract_version.to_string(),
            dependencies: DEPENDENCIES.iter().map(Into::into).collect(),
            metadata: metadata.map(Into::into),
        };
        assert_eq!(expected, loaded);
    }

    #[test]
    fn module_upgrade() {
        let mut store = MockStorage::new();
        let contract_name = "abstract:manager";
        let contract_version = "0.19.2";
        cw2::CONTRACT
            .save(
                &mut store,
                &ContractVersion {
                    contract: contract_name.to_owned(),
                    version: contract_version.to_owned(),
                },
            )
            .unwrap();

        // Patch upgrade
        let to_version = "0.19.3".parse().unwrap();
        let res = assert_contract_upgrade(&store, contract_name, to_version);
        assert!(res.is_ok());

        // Minor upgrade
        let to_version = "0.20.0".parse().unwrap();
        let res = assert_contract_upgrade(&store, contract_name, to_version);
        assert!(res.is_ok());

        // Minor with patch upgrade
        let to_version = "0.20.1".parse().unwrap();
        let res = assert_contract_upgrade(&store, contract_name, to_version);
        assert!(res.is_ok());

        // Major upgrade
        let to_version = "1.0.0".parse().unwrap();
        let res = assert_contract_upgrade(&store, contract_name, to_version);
        assert!(res.is_ok());
    }

    #[test]
    fn module_upgrade_err() {
        let mut store = MockStorage::new();
        let contract_name = "abstract:manager";
        let contract_version = "0.19.2";
        cw2::CONTRACT
            .save(
                &mut store,
                &ContractVersion {
                    contract: contract_name.to_owned(),
                    version: contract_version.to_owned(),
                },
            )
            .unwrap();

        // Downgrade
        let to_version: Version = "0.19.1".parse().unwrap();
        let err = assert_contract_upgrade(&store, contract_name, to_version.clone()).unwrap_err();
        assert_eq!(
            err,
            AbstractError::CannotDowngradeContract {
                contract: contract_name.to_string(),
                from: contract_version.parse().unwrap(),
                to: to_version
            }
        );

        // Minor upgrade
        let to_version: Version = "0.21.0".parse().unwrap();
        let err = assert_contract_upgrade(&store, contract_name, to_version.clone()).unwrap_err();
        assert_eq!(
            err,
            AbstractError::CannotSkipVersion {
                contract: contract_name.to_string(),
                from: contract_version.parse().unwrap(),
                to: to_version
            }
        );

        // Major upgrade
        let to_version: Version = "2.0.0".parse().unwrap();
        let err = assert_contract_upgrade(&store, contract_name, to_version.clone()).unwrap_err();
        assert_eq!(
            err,
            AbstractError::CannotSkipVersion {
                contract: contract_name.to_string(),
                from: contract_version.parse().unwrap(),
                to: to_version
            }
        );
    }
}
