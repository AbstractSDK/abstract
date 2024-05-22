mod module;
mod ping_callback;
mod rematch;

pub const PING_CALLBACK: &str = "ping_callback";
pub const QUERY_PROXY_CONFIG_CALLBACK: &str = "proxy_config";
pub const REMOTE_PREVIOUS_PING_PONG_CALLBACK: &str = "remote_rpp";

pub use module::receive_module_ibc;
pub use ping_callback::ping_callback;
pub use rematch::{proxy_config, rematch_ping_pong};
