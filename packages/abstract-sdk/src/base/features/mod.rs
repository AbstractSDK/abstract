//! # Module Features
//! Features are traits that are implemented on the base layer of a module. Implementing a feature unlocks the API objects that are dependent on it.  
//! You can easily create and provide your own API for other smart-contract developers by using these features as trait bounds.

/// These are very low-level traits that are implemented on the object. The apis depend on these features to be implemented.
mod abstract_name_service;
mod abstract_registry_access;
mod dependencies;
mod identification;
mod module_identification;

pub use crate::apis::respond::AbstractResponse;
pub use abstract_name_service::AbstractNameService;
pub use abstract_registry_access::AbstractRegistryAccess;
pub use dependencies::Dependencies;
pub use identification::Identification;
pub use module_identification::ModuleIdentification;
