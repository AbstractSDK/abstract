pub mod contract;
pub mod error;
pub mod handlers;
pub mod msg;
pub mod replies;
pub mod state;

pub use error::MyStandaloneError;

pub(crate) use contract::MY_STANDALONE;
/// The version of your standalone
pub const STANDALONE_VERSION: &str = env!("CARGO_PKG_VERSION");

pub use contract::interface::MyStandaloneInterface;

pub const MY_NAMESPACE: &str = "yournamespace";
pub const MY_STANDALONE_NAME: &str = "my-app";
pub const MY_STANDALONE_ID: &str = const_format::formatcp!("{MY_NAMESPACE}:{MY_STANDALONE_NAME}");
