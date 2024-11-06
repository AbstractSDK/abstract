#![cfg(feature = "interchain")]
use abstract_client::AbstractClient;
use abstract_client::GovernanceDetails;
use abstract_interface::IbcClient;
use cw_orch::mock::MockBase;
use cw_orch_interchain::prelude::*;

#[test]
fn create_remote_account() -> anyhow::Result<()> {
    let mock_interchain =
        MockBech32InterchainEnv::new(vec![("juno-1", "juno"), ("osmo-1", "osmo")]);

    let mock_juno = mock_interchain.get_chain("juno-1")?;
    let mock_osmo = mock_interchain.get_chain("osmo-1")?;

    let juno_abstr = AbstractClient::builder(mock_juno.clone()).build()?;
    let osmo_abstr = AbstractClient::builder(mock_osmo.clone()).build()?;

    juno_abstr.connect_to(&osmo_abstr, &mock_interchain)?;

    let juno_account = juno_abstr
        .account_builder()
        .install_adapter::<IbcClient<MockBase>>()
        .build()?;
    let remote_osmo_account = juno_account
        .remote_account_builder(mock_interchain.clone(), &osmo_abstr)
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
