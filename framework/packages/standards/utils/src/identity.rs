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
    let mut decomposed = platform_name.splitn(2, '>');
    match (
        decomposed.next().unwrap().to_owned(),
        decomposed.next().map(str::to_string),
    ) {
        (chain_name, Some(platform_name)) => (Some(chain_name), platform_name),
        (platform_name, None) => (None, platform_name),
    }
}

fn get_chain_name<'a>(env: &'a Env) -> &'a str {
    env.block.chain_id.rsplitn(2, '-').last().unwrap()
}

/// Helper to verify the DEX called is on the right chain
pub fn is_current_chain(env: &Env, chain_name: &str) -> bool {
    get_chain_name(env) == chain_name
}

/// Helper to verify the DEX called is on the right chain
pub fn is_available_on(platform: Box<dyn Identify>, env: &Env, chain_name: Option<&str>) -> bool {
    if let Some(chain_name) = chain_name {
        platform.is_available_on(chain_name)
    } else {
        let chain_name = get_chain_name(env);
        platform.is_available_on(chain_name)
    }
}
