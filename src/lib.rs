pub mod contract;
pub mod dependencies;
pub mod error;
mod handlers;
mod replies;
#[cfg(feature = "interface")]
pub mod interface;
pub mod msg;
pub mod state;

pub const TEMPLATE_MOD_ID: &str = "yournamespace:template";
