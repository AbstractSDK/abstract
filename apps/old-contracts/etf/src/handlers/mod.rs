// pub mod execute;
// pub mod instantiate;
// pub mod migrate;
// pub mod query;
pub mod execute;
pub mod instantiate;
pub mod query;
pub mod receive;
pub mod reply;

pub use crate::handlers::{
    execute::execute_handler, instantiate::instantiate_handler, query::query_handler,
    receive::receive_cw20, reply::*,
};
