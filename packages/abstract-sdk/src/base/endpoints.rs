mod execute;
mod ibc_callback;
mod instantiate;
pub(crate) mod migrate;
mod query;
mod receive;
mod reply;

// Provide endpoints under ::base::traits::
pub use execute::ExecuteEndpoint;
pub use ibc_callback::IbcCallbackEndpoint;
pub use instantiate::InstantiateEndpoint;
pub use migrate::MigrateEndpoint;
pub use query::QueryEndpoint;
pub use receive::ReceiveEndpoint;
pub use reply::ReplyEndpoint;
