mod callback;
mod module;
mod rematch;

pub const PING_CALLBACK: &str = "ping_callback";
pub const QUERY_PROXY_CONFIG_CALLBACK: &str = "proxy_config";
pub const REMOTE_PREVIOUS_PING_PONG_CALLBACK: &str = "remote_rpp";

#[cosmwasm_schema::cw_serde]
pub enum PingPongCallbacks {
    Ping,
    QueryProxyConfig,
    RemotePingPong,
}

pub use callback::ibc_callback;
pub use module::receive_module_ibc;
pub use rematch::{proxy_config, rematch_ping_pong};
