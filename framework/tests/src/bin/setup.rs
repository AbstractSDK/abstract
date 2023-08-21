use abstract_core::abstract_ica::IBC_APP_VERSION;
use abstract_core::ibc_client::{ExecuteMsgFns, QueryMsgFns};
use abstract_core::ibc_host::InstantiateMsg;
use abstract_core::objects::chain_name::ChainName;
use abstract_core::{IBC_CLIENT, IBC_HOST};

use abstract_interface::{Abstract, AccountFactoryExecFns, IbcClient, IbcClientExecFns, IbcHost};
use abstract_interface_integration_tests::ibc::set_env;
use abstract_interface_integration_tests::{JUNO, OSMOSIS};
use anyhow::Result as AnyResult;

use cw_orch::deploy::Deploy;
use cw_orch::prelude::*;

use clap::Parser;
use cw_orch::prelude::interchain_channel_builder::InterchainChannelBuilder;
use cw_orch::starship::Starship;
use cw_orch::state::ChainState;
use cw_orch_polytone::Polytone;

#[derive(Parser, Debug)]
struct Cli {
    skip_abstract_upload: Option<bool>,
}

fn deploy_on_one_chain(chain: &Daemon) -> AnyResult<()> {
    let args = Cli::parse();

    let chain_abstr = if args.skip_abstract_upload.unwrap_or(false) {
        Abstract::load_from(chain.clone())?
    } else {
        Abstract::deploy_on(chain.clone(), chain.sender().to_string())?
    };

    // now deploy IBC stuff
    let client = IbcClient::new(IBC_CLIENT, chain.clone());
    let host = IbcHost::new(IBC_HOST, chain.clone());
    client.upload()?;
    host.upload()?;

    client.instantiate(
        &abstract_core::ibc_client::InstantiateMsg {
            ans_host_address: chain_abstr.ans_host.addr_str()?,
            chain: chain.state().0.chain_data.chain_id.to_string(),
            version_control_address: chain_abstr.version_control.addr_str()?,
        },
        None,
        None,
    )?;

    // Client needs to be registered as a module to the abstract core
    chain_abstr.version_control.register_adapters(vec![(
        client.as_instance(),
        ibc_client::contract::CONTRACT_VERSION.to_string(),
    )])?;

    host.instantiate(
        &InstantiateMsg {
            ans_host_address: chain_abstr.ans_host.addr_str()?,
            account_factory_address: chain_abstr.account_factory.addr_str()?,
            version_control_address: chain_abstr.version_control.addr_str()?,
        },
        None,
        None,
    )?;

    // We need to register the ibc host in the distant chain account factory
    chain_abstr
        .account_factory
        .update_config(None, Some(host.address().unwrap().to_string()), None, None)
        .unwrap();

    Ok(())
}

fn deploy_contracts(juno: &Daemon, osmosis: &Daemon) -> anyhow::Result<()> {
    deploy_on_one_chain(juno)?;
    deploy_on_one_chain(osmosis)?;
    Ok(())
}

async fn create_channel(
    contract1: &dyn ContractInstance<Daemon>,
    contract2: &dyn ContractInstance<Daemon>,
    starship: &Starship,
) -> AnyResult<()> {
    log::info!(
        "Start creating IBC connection between {} and {}",
        contract1.address()?,
        contract2.address()?
    );

    InterchainChannelBuilder::default()
        .from_contracts(contract1, contract2)
        .create_channel(starship.client(), IBC_APP_VERSION)
        .await?;

    let connection = starship
        .client()
        .registry()
        .await
        .ibc_path(
            contract1.get_chain().state().0.chain_data.chain_id.as_str(),
            contract2.get_chain().state().0.chain_data.chain_id.as_str(),
        )
        .await?
        .chain_1
        .connection_id;

    log::info!(
        "Successfully created a channel between {} and {} on connection '{}'",
        contract1.address().unwrap(),
        contract2.address().unwrap(),
        connection.as_str(),
    );

    Ok(())
}

fn join_host_and_clients(
    polytone: &Polytone<Daemon>,
    chain1: &Daemon,
    chain2: &Daemon,
    rt: &tokio::runtime::Runtime,
    starship: &Starship,
) -> AnyResult<()> {
    let client = IbcClient::new(IBC_CLIENT, chain1.clone());
    let host = IbcHost::new(IBC_HOST, chain2.clone());

    // First we register client and host respectively
    let chain1_name = chain1.state().0.chain_data.chain_name.to_string();
    let chain1_id = chain1.state().0.chain_data.chain_id.to_string();
    let chain2_name = chain2.state().0.chain_data.chain_name.to_string();

    let proxy_tx_result = client.register_chain_host(
        chain2_name.into(),
        host.address()?.to_string(),
        polytone.note.address()?.to_string(),
    )?;

    // We make sure the proxy address is saved
    rt.block_on(
        polytone
            .channel
            .await_ibc_execution(chain1_id, proxy_tx_result.txhash),
    )?;

    let proxy_address = client.host(chain2_name)?;

    host.register_chain_proxy(chain1_name.into(), proxy_address.remote_polytone_proxy)?;

    rt.block_on(create_channel(&client, &host, starship))?;
    Ok(())
}

fn ibc_abstract_setup() -> AnyResult<()> {
    set_env();

    // Chains setup
    let rt: tokio::runtime::Runtime = tokio::runtime::Runtime::new().unwrap();

    let starship = Starship::new(rt.handle().to_owned(), None)?;
    let interchain: InterchainEnv = starship.interchain_env();

    let juno = interchain.daemon(JUNO)?;
    let osmosis = interchain.daemon(OSMOSIS)?;

    // Deploying abstract and the IBC abstract logic
    deploy_contracts(&juno, &osmosis)?;

    let polytone = cw_orch_polytone::deploy(&rt, &starship, JUNO, OSMOSIS, "1".to_string())?;

    // Create the connection between client and host
    join_host_and_clients(&polytone, &osmosis, &juno, &rt, &starship)?;

    // Some tests to make sure the connection has been established between the 2 contracts
    // We query the channels for each host to see if the client has been connected
    let osmosis_client = IbcClient::new(IBC_CLIENT, osmosis);

    let osmosis_channels: abstract_core::ibc_client::ListRemoteHostsResponse =
        osmosis_client.list_remote_hosts()?;

    assert_eq!(osmosis_channels.hosts[0].0, ChainName::from("juno"));

    Ok(())
}

fn main() {
    env_logger::init();
    ibc_abstract_setup().unwrap();
}
