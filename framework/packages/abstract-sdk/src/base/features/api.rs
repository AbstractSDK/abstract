use cosmwasm_std::Deps;

use super::ModuleIdentification;

/// Return the identifier for this api.
pub trait ApiIdentification {
    /// Get the api identifier.
    fn api_id() -> String;
}

pub trait AbstractApi<T: ModuleIdentification> {
    fn base(&self) -> &T;
    fn deps(&self) -> Deps;
}
