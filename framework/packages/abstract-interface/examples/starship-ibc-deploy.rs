use abstract_interface::Abstract;
use cw_orch::prelude::*;
use cw_orch_interchain::prelude::*;

pub const ABSTRACT_VERSION: &str = env!("CARGO_PKG_VERSION");

fn full_deploy() -> cw_orch::anyhow::Result<()> {
    let starship = Starship::new(None)?;
    let interchain = starship.interchain_env();

    let src_chain = interchain.get_chain("juno-1")?;
    let dst_chain = interchain.get_chain("stargaze-1")?;

    let src_abstr = Abstract::deploy_on(src_chain.clone(), src_chain.sender_addr().to_string())?;
    let dst_abstr = Abstract::deploy_on(dst_chain.clone(), dst_chain.sender_addr().to_string())?;

    src_abstr.connect_to(&dst_abstr, &interchain)?;

    Ok(())
}

fn main() {
    dotenv().ok();
    env_logger::init();

    use dotenv::dotenv;

    full_deploy().unwrap();
}
