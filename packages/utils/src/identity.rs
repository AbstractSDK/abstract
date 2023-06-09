use cosmwasm_std::Env;

pub trait Identify {
    /// This should return wether the platform is available on the chain designated by chain_name
    /// For instance, Wyndex is available on juno-1, so wyndex.is_available_on("juno") should return true
    /// We will only pass the chain name and never the chain_id to this function
    fn is_available_on(&self, chain_name: &str) -> bool;
    fn name(&self) -> &'static str;
}

/// Helper to un-nest the platform name
/// The platform_name has format juno>wyndex 
pub fn decompose_platform_name(platform_name: String) -> (String, String){
    let decomposed: Vec<_> = platform_name.splitn(2, '>').collect();
    (decomposed[0].to_string(), decomposed[1].to_string())
}

/// Helper to verify the DEX called is on the right chain
pub fn is_current_chain(env: Env, chain_name: String) -> bool{
    env.block.chain_id.rsplitn(2, '-').collect::<Vec<_>>()[1] == chain_name
}