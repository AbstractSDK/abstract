pub mod execute;
pub mod instantiate;
pub mod query;
pub mod nois_callback;

pub use crate::handlers::{
    execute::execute_handler, instantiate::instantiate_handler, query::query_handler, nois_callback::nois_callback_handler,
};
