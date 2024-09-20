#![cfg(feature = "interchain")]
use abstract_client::AbstractClient;
use abstract_client::GovernanceDetails;
use abstract_interface::IbcClient;
use abstract_testing::prelude::mock_bech32_admin;
use cw_orch::mock::MockBase;
use cw_orch::prelude::*;
use cw_orch_interchain::prelude::*;
use cw_orch_interchain::MockBech32InterchainEnv;

#[test]
fn create_remote_account() -> anyhow::Result<()> {
    let mock_interchain =
        MockBech32InterchainEnv::new(vec![("juno-1", "juno"), ("osmo-1", "osmo")]);

    let mut mock_juno = mock_interchain.get_chain("juno-1")?;
    mock_juno.set_sender(mock_bech32_admin(&mock_juno));
    let mut mock_osmo = mock_interchain.get_chain("osmo-1")?;
    mock_osmo.set_sender(mock_bech32_admin(&mock_osmo));

    let juno_abstr =
        AbstractClient::builder(mock_juno.clone()).build(mock_juno.sender().clone())?;
    let osmo_abstr =
        AbstractClient::builder(mock_osmo.clone()).build(mock_osmo.sender().clone())?;

    juno_abstr.connect_to(&osmo_abstr, &mock_interchain)?;

    let juno_account = juno_abstr
        .account_builder()
        .install_adapter::<IbcClient<MockBase>>()?
        .build()?;
    let remote_osmo_account = juno_account
        .remote_account_builder(&mock_interchain, &osmo_abstr)
        .build()?;

    // Make sure it's created and remote
    let ownership = remote_osmo_account.ownership()?;
    let GovernanceDetails::External {
        governance_address: _,
        governance_type,
    } = ownership.owner
    else {
        panic!("unexpected governance details for remote account");
    };
    assert_eq!(governance_type, "abstract-ibc");
    Ok(())
}
