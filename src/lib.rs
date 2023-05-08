pub mod contract;
pub mod error;
mod handlers;
pub mod msg;
pub mod state;
pub mod dependencies;
#[cfg(feature = "interface")]
mod interface;

pub const TEMPLATE_ID: &str = "yournamespace:template";
