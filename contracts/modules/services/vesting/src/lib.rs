pub mod contract;
pub mod state;

#[cfg(test)]
#[cfg(not(target_arch = "wasm32"))]
mod tests;
