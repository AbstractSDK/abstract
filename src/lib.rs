pub mod contract;
pub mod error;
mod handlers;
#[cfg(feature = "interface")]
pub mod interface;
pub mod msg;
mod replies;
pub mod state;

#[cfg(feature = "interface")]
pub use interface::Template;
#[cfg(feature = "interface")]
pub use msg::{TemplateExecuteMsgFns, TemplateQueryMsgFns};
