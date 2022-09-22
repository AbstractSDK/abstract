pub mod contract;
mod queries;
pub mod state;
#[cfg(test)]
#[cfg(not(target_arch = "wasm32"))]
mod tests;
