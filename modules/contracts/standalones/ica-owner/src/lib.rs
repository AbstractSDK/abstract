pub mod contract;
pub mod error;
pub mod msg;
pub mod state;

use abstract_standalone::StandaloneContract;
use cosmwasm_std::Response;
pub use error::MyStandaloneError;

/// The version of your standalone
pub const STANDALONE_VERSION: &str = env!("CARGO_PKG_VERSION");

pub const MY_NAMESPACE: &str = "yournamespace";
pub const MY_STANDALONE_NAME: &str = "my-standalone";
pub const MY_STANDALONE_ID: &str = const_format::formatcp!("{MY_NAMESPACE}:{MY_STANDALONE_NAME}");

/// The type of the result returned by your standalone's entry points.
pub type MyStandaloneResult<T = Response> = Result<T, MyStandaloneError>;

/// The type of the standalone that is used to build your contract object and access the Abstract SDK features.
pub type MyStandalone = StandaloneContract;

pub const MY_STANDALONE: MyStandalone =
    MyStandalone::new(MY_STANDALONE_ID, STANDALONE_VERSION, None);

// cw-orch related interface
#[cfg(not(target_arch = "wasm32"))]
mod interface;

#[cfg(not(target_arch = "wasm32"))]
pub use interface::{ica_controller, MyStandaloneInterface};
