pub mod contract;
pub mod error;
pub mod msg;
pub mod state;

#[cfg(target_arch = "wasm32")]
cosmwasm_std::create_entry_points!(contract); // Makes the initialize, excecute and query entry points, can be done manualy with #[cfg_attr(not(feature = "library"), entry_point)]
