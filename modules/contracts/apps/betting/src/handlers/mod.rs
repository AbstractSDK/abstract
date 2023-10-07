// pub mod execute;
// pub mod instantiate;
// pub mod migrate;
// pub mod query;
pub mod execute;
pub mod instantiate;
pub mod query;

pub use crate::handlers::{
    execute::execute_handler, instantiate::instantiate_handler, query::query_handler,
};
