use abstract_core::ibc_client::QueryMsgFns;
use abstract_core::objects::chain_name::ChainName;
use abstract_core::IBC_CLIENT;

use abstract_interface::{Abstract, IbcClient};
use abstract_interface_integration_tests::ibc::{set_env, TEST_STARSHIP_CONFIG};
use abstract_interface_integration_tests::{JUNO, STARGAZE};
use anyhow::Result as AnyResult;

use cw_orch::deploy::Deploy;
use cw_orch::prelude::*;

use clap::Parser;
use cw_orch::starship::Starship;
use cw_orch_polytone::Polytone;

#[derive(Parser, Debug)]
struct Cli {
    skip_abstract_upload: Option<bool>,
}

/// Helper to choose wether to deploy abstract or load it from the chain
fn deploy_abstr(chain: &Daemon) -> AnyResult<Abstract<Daemon>> {
    let args = Cli::parse();

    let chain_abstr = if args.skip_abstract_upload.unwrap_or(false) {
        Abstract::load_from(chain.clone())?
    } else {
        Abstract::deploy_on(chain.clone(), chain.sender().to_string())?
    };

    Ok(chain_abstr)
}

/// Helper to choose wether to deploy abstract or load it from the chain
fn deploy_polytone(chain: &Daemon) -> AnyResult<Polytone<Daemon>> {
    let args = Cli::parse();

    let chain_polytone = if args.skip_abstract_upload.unwrap_or(false) {
        Polytone::load_from(chain.clone())?
    } else {
        Polytone::deploy_on(chain.clone(), None)?
    };

    Ok(chain_polytone)
}

fn ibc_abstract_setup() -> AnyResult<()> {
    set_env();

    // Chains setup
    let rt: tokio::runtime::Runtime = tokio::runtime::Runtime::new().unwrap();

    let config_path = format!("{}{}", env!("CARGO_MANIFEST_DIR"), TEST_STARSHIP_CONFIG);

    let starship = Starship::new(rt.handle().clone(), &config_path, None)?;
    let interchain: InterchainEnv = starship.interchain_env();

    let stargaze = interchain.daemon(STARGAZE)?;
    let juno = interchain.daemon(JUNO)?;

    // Deploying abstract and the IBC abstract logic
    let stargaze_abstr = deploy_abstr(&stargaze)?;
    let juno_abstr = deploy_abstr(&juno)?;

    // Deploying polytone on both chains
    let stargaze_polytone = deploy_polytone(&stargaze)?;
    let juno_polytone = deploy_polytone(&juno)?;

    // Creating a connection between 2 polytone deployments
    let polytone_account =
        cw_orch_polytone::deploy(&rt, &starship, &stargaze_polytone, &juno_polytone)?;

    // Create the connection between client and host
    stargaze_abstr.ibc_connection_with(&rt, &juno_abstr, &polytone_account)?;

    // Some tests to make sure the connection has been established between the 2 contracts
    // We query the channels for each host to see if the client has been connected
    let stargaze_client = IbcClient::new(IBC_CLIENT, stargaze);

    let stargaze_channels: abstract_core::ibc_client::ListRemoteHostsResponse =
        stargaze_client.list_remote_hosts()?;

    assert_eq!(stargaze_channels.hosts[0].0, ChainName::from("juno"));

    // We test creating a remote account ?

    Ok(())
}

fn main() {
    env_logger::init();
    ibc_abstract_setup().unwrap();
}
