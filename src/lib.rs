pub mod contract;
pub mod dependencies;
pub mod error;
mod handlers;
#[cfg(feature = "interface")]
pub mod interface;
pub mod msg;
pub mod state;

pub const TEMPLATE_ID: &str = "yournamespace:template";
