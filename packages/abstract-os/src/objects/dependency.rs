//! Dependency definitions for Abstract Modules

use semver::Comparator;
use serde::{Deserialize, Serialize};

use crate::manager::state::ModuleId;

/// Statically defined dependency used in-contract
#[derive(Debug, Clone)]
pub struct StaticDependency {
    pub id: ModuleId<'static>,
    pub version_req: &'static [Comparator],
}

impl StaticDependency {
    pub const fn new(
        module_id: ModuleId<'static>,
        version_requirement: &'static [Comparator],
    ) -> Self {
        Self {
            id: module_id,
            version_req: version_requirement,
        }
    }
}

/// On-chain stored version of the module dependencies. Retrievable though raw-queries.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct Dependency {
    pub id: String,
    pub version_req: Vec<Comparator>,
}

impl From<&StaticDependency> for Dependency {
    fn from(dep: &StaticDependency) -> Self {
        Self {
            id: dep.id.to_string(),
            version_req: dep.version_req.into(),
        }
    }
}
