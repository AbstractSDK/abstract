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
// Returns (Option<chain_id>, platform_name)
pub fn decompose_platform_name(platform_name: &str) -> (Option<String>, String) {
    let decomposed: Vec<_> = platform_name.splitn(2, '>').collect();
    if decomposed.len() == 1 {
        (None, decomposed[0].to_string())
    } else {
        (Some(decomposed[0].to_string()), decomposed[1].to_string())
    }
}

fn get_chain_name(env: Env) -> String {
    env.block.chain_id.rsplitn(2, '-').collect::<Vec<_>>()[1].to_string()
}

/// Helper to verify the DEX called is on the right chain
pub fn is_current_chain(env: Env, chain_name: &str) -> bool {
    get_chain_name(env) == chain_name
}

/// Helper to verify the DEX called is on the right chain
pub fn is_available_on(platform: Box<dyn Identify>, env: Env, chain_name: Option<&str>) -> bool {
    if let Some(chain_name) = chain_name {
        platform.is_available_on(chain_name)
    } else {
        let chain_name = get_chain_name(env);
        platform.is_available_on(&chain_name)
    }
}
