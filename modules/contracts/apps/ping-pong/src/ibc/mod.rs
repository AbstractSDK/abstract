mod callback;
mod module;

use abstract_app::objects::chain_name::ChainName;
pub use callback::ibc_callback;
pub use module::receive_module_ibc;

#[cosmwasm_schema::cw_serde]
pub enum PingPongIbcCallback {
    // Callback
    RematchQuery { rematch_chain: ChainName },
}
