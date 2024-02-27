use abstract_client::AbstractClient;
use abstract_interface::Abstract;
use cosmwasm_std::Addr;
use cw_orch::prelude::*;
use cw_orch::{contract::Deploy, daemon::ChainInfo};
use cw_orch_clone_testing::CloneTesting;

/// Returns a shared tokio runtime for all tests
pub fn rt() -> &'static tokio::runtime::Runtime {
    lazy_static::lazy_static! {
        static ref RT: tokio::runtime::Runtime = tokio::runtime::Builder::new_multi_thread()
        .enable_all()
        .build()
        .expect("Should create a tokio runtime");
    }
    &RT
}

/// Sets up the CloneTesting for chain.
/// Returns the abstract deployment and sender (=mainnet admin)
pub fn setup(
    chain: ChainInfo,
    sender: &str,
) -> anyhow::Result<(Abstract<CloneTesting>, CloneTesting)> {
    let _ = env_logger::builder().is_test(true).try_init();
    let sender = Addr::unchecked(sender);
    // Run migration tests against Juno mainnet
    // We set the state file to be able to clone test
    std::env::set_var("STATE_FILE", "../scripts/state.json");
    let mut app = CloneTesting::new(rt(), chain)?;
    app.set_sender(sender.clone());

    let abstr_deployment = Abstract::load_from(app.clone())?;
    Ok((abstr_deployment, app))
}

/// Sets up the CloneTesting for chain.
/// Returns the abstract client and sender (=mainnet admin)
pub fn deploy_client(
    chain: ChainInfo,
    sender: &str,
) -> anyhow::Result<(AbstractClient<CloneTesting>, CloneTesting)> {
    let _ = env_logger::builder().is_test(true).try_init();
    // Run migration tests against Juno mainnet
    // We set the state file to be able to clone test
    std::env::set_var("STATE_FILE", "../scripts/state.json");
    let sender = Addr::unchecked(sender);
    let mut app = CloneTesting::new(rt(), chain)?;
    app.set_sender(sender.clone());

    let abstr_deployment = AbstractClient::builder(app.clone()).build()?;
    Ok((abstr_deployment, app))
}
