pub mod custom;
pub mod execute;
pub mod instantiate;
pub mod query;
pub mod receive;

pub use crate::handlers::{
    execute::execute_handler, instantiate::instantiate_handler, query::query_handler,
    receive::receive_cw20,
};
