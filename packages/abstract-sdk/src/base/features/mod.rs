//! # Module Features
//! Features are traits that are implemented on the base layer of a module. Implementing a feature unlocks the API objects that are dependent on it.  
//! You can easily create and provide your own API for other smart-contract developers by using these features as trait bounds.

mod abstract_name_service;
mod dependencies;
mod identification;
mod module_identification;
mod register_access;
mod respond;

pub use abstract_name_service::AbstractNameService;
pub use dependencies::Dependencies;
pub use identification::Identification;
pub use module_identification::ModuleIdentification;
pub use register_access::RegisterAccess;
pub use respond::AbstractResponse;
