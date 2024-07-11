use abstract_interface::Abstract;
use cw_orch::prelude::*;
use cw_orch_interchain::prelude::*;

pub const ABSTRACT_VERSION: &str = env!("CARGO_PKG_VERSION");

fn full_deploy() -> cw_orch::anyhow::Result<()> {
    let interchain = MockBech32InterchainEnv::new(vec![("src-1", "src"), ("dst-1", "dst")]);

    let src_chain = interchain.chain("src-1")?;
    let dst_chain = interchain.chain("dst-1")?;

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
