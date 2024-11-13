pub mod execute;
pub mod instantiate;
pub mod query;
pub mod sudo;

pub use crate::handlers::{
    execute::execute_handler, instantiate::instantiate_handler, query::query_handler,
    sudo::sudo_handler,
};

pub const ICS20_CALLBACK_ID: u64 = 1;
