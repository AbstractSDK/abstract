#![cfg(feature = "interchain")]
use abstract_client::AbstractClient;
use abstract_client::AbstractInterchainClient;
use abstract_client::Environment;
use abstract_client::GovernanceDetails;
use abstract_interface::Abstract;
use abstract_interface::IbcClient;
use abstract_std::ibc_client::QueryMsgFns as _;
use abstract_std::ibc_host::QueryMsgFns as _;
use cw_orch::contract::Deploy;
use cw_orch::mock::MockBase;
use cw_orch_interchain::prelude::*;
use cw_orch_interchain::MockBech32InterchainEnv;

#[test]
fn create_remote_account() -> anyhow::Result<()> {
    let mock_interchain =
        MockBech32InterchainEnv::new(vec![("juno-1", "juno"), ("osmo-1", "osmo")]);

    let mock_juno = mock_interchain.chain("juno-1")?;
    let mock_osmo = mock_interchain.chain("osmo-1")?;

    let juno_abstr = AbstractClient::builder(mock_juno.clone()).build()?;
    let osmo_abstr = AbstractClient::builder(mock_osmo.clone()).build()?;

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

#[test]
fn create_multiple_abstract() -> anyhow::Result<()> {
    let chain_number = 4;
    let interchain = MockBech32InterchainEnv::new(vec![
        ("osmosis-1", "osmo"),
        ("juno-1", "juno"),
        ("archway-1", "archway"),
        ("neutron-1", "neutron"),
    ]);

    let mut builder = AbstractInterchainClient::builder();

    for (_, mock) in interchain.mocks.iter() {
        builder.chain(mock.clone());
    }

    builder.post_setup_function(|_| Ok(()));

    let all_abstr = builder.build(&interchain)?;

    // We make sure all chains are connected to n-1 chains
    assert!(all_abstr.into_iter().all(|(_, abstr)| {
        let all_hosts = abstr.ibc_client().list_remote_hosts().unwrap();

        let all_clients = Abstract::load_from(abstr.environment())
            .unwrap()
            .ibc
            .host
            .client_proxies(None, None)
            .unwrap();

        all_hosts.hosts.len() == chain_number - 1 && all_clients.chains.len() == chain_number - 1
    }));

    Ok(())
}

#[test]
fn error_in_post_error() -> anyhow::Result<()> {
    let interchain = MockBech32InterchainEnv::new(vec![
        ("osmosis-1", "osmo"),
        ("juno-1", "juno"),
        ("archway-1", "archway"),
        ("neutron-1", "neutron"),
    ]);

    let mut builder = AbstractInterchainClient::builder();

    for (_, mock) in interchain.mocks.iter() {
        builder.chain(mock.clone());
    }

    builder.post_setup_function(|e| {
        if e.environment().chain_id() == "osmosis-1" {
            Err(abstract_client::AbstractClientError::FundsWithAutoFund {})
        } else {
            Ok(())
        }
    });

    assert!(builder.build(&interchain).is_err());

    Ok(())
}
