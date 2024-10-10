use cosmwasm_std::Deps;

use crate::features::ModuleIdentification;

pub trait AbstractApi<T: ModuleIdentification> {
    const API_ID: &'static str;
    fn base(&self) -> &T;
    fn deps(&self) -> Deps;
    /// Get the api identifier.
    fn api_id() -> String {
        Self::API_ID.to_owned()
    }
}

#[cfg(test)]
pub(crate) mod test {
    use super::*;

    pub fn abstract_api_test<Base: ModuleIdentification, T: AbstractApi<Base>>(api: T) {
        let api_id = T::api_id();
        assert_eq!(api_id, T::API_ID);
        let base = api.base();
        assert_eq!(base.module_id(), abstract_testing::module::TEST_MODULE_ID);
        let deps = api.deps();
        assert!(deps
            .api
            .addr_validate(
                cosmwasm_std::testing::MockApi::default()
                    .addr_make("test")
                    .as_str()
            )
            .is_ok())
    }
}
