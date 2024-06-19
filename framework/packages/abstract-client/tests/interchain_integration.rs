#![cfg(feature = "interchain")]
use abstract_client::AbstractClient;
use abstract_client::GovernanceDetails;
use abstract_interface::IbcClient;
use abstract_polytone::handshake::POLYTONE_VERSION;
use cw_orch::mock::MockBase;
use cw_orch::prelude::*;
use cw_orch_interchain::prelude::*;
use cw_orch_interchain::MockBech32InterchainEnv;
use cw_orch_polytone::Polytone;

#[test]
fn create_remote_account() -> anyhow::Result<()> {
    let mock_interchain =
        MockBech32InterchainEnv::new(vec![("juno-1", "juno"), ("osmo-1", "osmo")]);

    let mock_juno = mock_interchain.chain("juno-1")?;
    let mock_osmo = mock_interchain.chain("osmo-1")?;

    let juno_abstr = AbstractClient::builder(mock_juno.clone()).build()?;
    let osmo_abstr = AbstractClient::builder(mock_osmo.clone()).build()?;

    // Deploying polytone on both chains
    let polytone_juno = Polytone::deploy_on(mock_juno, None)?;
    let polytone_osmo = Polytone::deploy_on(mock_osmo, None)?;

    // Creating a connection between 2 polytone deployments
    mock_interchain.create_contract_channel(
        &polytone_juno.note,
        &polytone_osmo.voice,
        POLYTONE_VERSION,
        None,
    )?;
    juno_abstr.ibc_connection_with(&osmo_abstr, &mock_interchain)?;

    let juno_account = juno_abstr
        .account_builder()
        .install_adapter::<IbcClient<MockBase>>()?
        .build()?;
    let remote_osmo_account = juno_account
        .remote_account_builder(&mock_interchain, &osmo_abstr)
        .build()?;

    // Make sure it's created and remote
    let info = remote_osmo_account.info()?;
    let GovernanceDetails::External {
        governance_address: _,
        governance_type,
    } = info.governance_details
    else {
        panic!("unexpected governance details for remote account");
    };
    assert_eq!(governance_type, "abstract-ibc");
    Ok(())
}
