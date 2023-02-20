//! Dependency definitions for Abstract Modules
use cw_semver::{Comparator, Version};
use serde::{Deserialize, Serialize};

use crate::ModuleId;

/// Statically defined dependency used in-contract
#[derive(Debug, Clone, PartialEq)]
pub struct StaticDependency {
    pub id: ModuleId<'static>,
    pub version_req: &'static [&'static str],
}

impl StaticDependency {
    pub const fn new(
        module_id: ModuleId<'static>,
        version_requirement: &'static [&'static str],
    ) -> Self {
        Self {
            id: module_id,
            version_req: version_requirement,
        }
    }

    /// Iterate through the (statically provided) version requirements and ensure that they are valid.
    pub fn check(&self) -> Result<(), cw_semver::Error> {
        for req in self.version_req {
            Comparator::parse(req)?;
        }
        Ok(())
    }

    /// Iterates through the version requirements and checks that the provided **version** is compatible.
    pub fn matches(&self, version: &Version) -> bool {
        self.version_req
            .iter()
            .all(|req| Comparator::parse(req).unwrap().matches(version))
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
            version_req: dep.version_req.iter().map(|s| s.parse().unwrap()).collect(),
        }
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use speculoos::prelude::*;
    use std::borrow::Borrow;

    #[test]
    fn test_static_constructor() {
        const VERSION_CONSTRAINT: [&str; 1] = ["^1.0.0"];

        let dep = StaticDependency::new("test", &VERSION_CONSTRAINT);

        assert_that!(dep.id).is_equal_to("test");
        assert_that!(&dep.version_req.to_vec()).is_equal_to(VERSION_CONSTRAINT.to_vec());
    }

    #[test]
    fn static_check_passes() {
        const VERSION_CONSTRAINT: [&str; 1] = ["^1.0.0"];

        let dep = StaticDependency::new("test", &VERSION_CONSTRAINT);

        assert_that!(dep.check()).is_ok();
    }

    #[test]
    fn static_check_fails() {
        const VERSION_CONSTRAINT: [&str; 1] = ["^1e.0"];

        let dep = StaticDependency::new("test", &VERSION_CONSTRAINT);

        assert_that!(dep.check()).is_err();
    }

    #[test]
    fn matches_should_match_matching_versions() {
        const VERSION_CONSTRAINT: [&str; 1] = ["^1.0.0"];

        let dep = StaticDependency::new("test", &VERSION_CONSTRAINT);

        assert_that!(dep.matches(&Version::parse("1.0.0").unwrap())).is_true();
        assert_that!(dep.matches(&Version::parse("1.1.0").unwrap())).is_true();
        assert_that!(dep.matches(&Version::parse("1.1.1").unwrap())).is_true();
    }

    #[test]
    fn matches_should_not_match_non_matching_versions() {
        const VERSION_CONSTRAINT: [&str; 1] = ["^1.0.0"];

        let dep = StaticDependency::new("test", &VERSION_CONSTRAINT);

        assert_that!(dep.matches(&Version::parse("2.0.0").unwrap())).is_false();
        assert_that!(dep.matches(&Version::parse("0.1.0").unwrap())).is_false();
        assert_that!(dep.matches(&Version::parse("0.1.1").unwrap())).is_false();
    }

    #[test]
    fn test_dependency_from_static() {
        const VERSION_CONSTRAINT: [&str; 1] = ["^1.0.0"];

        let dep = StaticDependency::new("test", &VERSION_CONSTRAINT);

        let dep: Dependency = dep.borrow().into();

        assert_that!(dep.id).is_equal_to("test".to_string());
        assert_that!(dep.version_req).is_equal_to(vec![Comparator::parse("^1.0.0").unwrap()]);
    }
}
