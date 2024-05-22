mod module;
mod ping_callback;

pub const PING_CALLBACK: &str = "ping_callback";
pub use module::receive_module_ibc;
pub use ping_callback::ping_callback;
