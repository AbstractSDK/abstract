use abstract_sdk::core::objects::dependency::StaticDependency;

const EXAMPLE_DEP: StaticDependency = StaticDependency::new("example:dep", &[">=0.3.0"]);

/// Dependencies for the app
pub const TEMPLATE_DEPS: &[StaticDependency] = &[EXAMPLE_DEP];

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_dependencies() {
        TEMPLATE_DEPS.iter().for_each(|dep| {
            dep.check().unwrap();
        });
    }
}
