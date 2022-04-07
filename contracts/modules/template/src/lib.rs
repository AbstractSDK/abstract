mod commands;
pub mod contract;
pub mod dapp_base;
pub mod msg;

#[cfg(test)]
#[cfg(not(target_arch = "wasm32"))]
pub mod tests;
