pub trait Identify {
    /// This should return wether the platform is available on the chain designated by chain_name
    /// For instance, Wyndex is available on juno-1, so wyndex.is_available_on("juno") should return true
    /// We will only pass the chain name and never the chain_id to this function
    fn is_available_on(&self, chain_name: &str) -> bool;
    fn name(&self) -> &'static str;
}
