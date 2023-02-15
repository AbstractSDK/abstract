#![doc(html_logo_url = "https://raw.githubusercontent.com/Abstract-OS/assets/mainline/logo.svg")]
#![doc = include_str!("../README.md")]
#![doc(test(attr(
    warn(unused),
    deny(warnings),
    // W/o this, we seem to get some bogus warning about `extern crate zbus`.
    allow(unused_extern_crates, unused),
)))]

pub type AbstractSdkResult<T> = Result<T, crate::error::AbstractSdkError>;

pub extern crate abstract_macros as macros;
pub extern crate abstract_os as os;

mod ans_resolve;
mod apis;

pub mod base;
pub mod cw_helpers;
mod error;
pub mod feature_objects;

pub use error::{AbstractSdkError, EndpointError};

pub use crate::apis::{
    bank::TransferInterface, dex::DexInterface, execution::Execution, ibc::IbcInterface,
    modules::ModuleInterface, respond::AbstractResponse, vault::VaultInterface,
    verify::OsVerification, version_registry::ModuleRegistryInterface,
};

pub mod features {
    //! # Feature traits
    //! Features are traits that are implemented on the base layer of a module. Implementing a feature unlocks the API objects that are dependent on it.  
    //!
    //! You can easily create and provide your own API for other smart-contract developers by using these features as trait bounds.
    pub use crate::base::features::*;
}

pub use ans_resolve::Resolve;

/// Common state-store namespaces.
pub mod namespaces {
    pub use abstract_os::objects::common_namespace::*;
}

/// Abstract reserved version control entries.
pub mod register {
    pub use abstract_os::registry::*;
}
