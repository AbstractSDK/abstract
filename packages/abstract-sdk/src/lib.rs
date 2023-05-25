#![doc(html_logo_url = "https://raw.githubusercontent.com/AbstractSDK/assets/mainline/logo.svg")]
#![doc = include_str ! ("../README.md")]
// #![doc(test(attr(warn(unused), deny(warnings), allow(unused_extern_crates, unused),)))]
#![warn(missing_docs)]

/// Result returned by the Abstract SDK APIs and features.
pub type AbstractSdkResult<T> = Result<T, crate::error::AbstractSdkError>;

pub extern crate abstract_core as core;

mod ans_resolve;
mod apis;

pub mod base;
pub mod cw_helpers;
mod error;
pub mod feature_objects;
pub mod prelude;

pub use error::{AbstractSdkError, EndpointError};

pub use crate::apis::{
    accounting::*, adapter::*, app::*, bank::*, execution::*, ibc::*, modules::*, respond::*,
    verify::*, version_registry::*,
};

#[cfg(feature = "stargate")]
pub use crate::apis::{distribution::*, grant::*};

pub mod features {
    //! # Feature traits
    //! Features are traits that are implemented on the base layer of a module. Implementing a feature unlocks the API objects that are dependent on it.
    //!
    //! You can easily create and provide your own Adapter for other smart-contract developers by using these features as trait bounds.
    pub use crate::base::features::*;
}

pub use ans_resolve::Resolve;

/// Common state-store namespaces.
pub mod namespaces {
    pub use abstract_core::objects::common_namespace::*;
}

/// Abstract reserved version control entries.
pub mod register {
    pub use abstract_core::registry::*;
}

#[cfg(feature = "test-utils")]
pub mod mock_module;
