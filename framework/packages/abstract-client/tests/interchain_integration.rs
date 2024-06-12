#![cfg(feature = "interchain")]
use cw_orch_interchain::MockBech32InterchainEnv;

#[test]
fn create_remote_account() -> anyhow::Result<()> {
    let _ibc = MockBech32InterchainEnv::new(vec![("juno-1", "juno"), ("osmo-1", "osmo")]);

    Ok(())
}
