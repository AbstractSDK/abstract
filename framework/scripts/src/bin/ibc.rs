use abstract_core::{ibc_client, IBC_CLIENT};
use abstract_interface::{Abstract, IbcClient, IbcHost};
use cw_orch::{
    deploy::Deploy,
    prelude::{interchain_channel_builder::InterchainChannelBuilder, *},
    starship::Starship,
};

const JUNO: &str = "juno-1";
const OSMOSIS: &str = "osmosis-2";

pub fn script() -> anyhow::Result<()> {
    let rt: tokio::runtime::Runtime = tokio::runtime::Runtime::new().unwrap();

    let starship = Starship::new(rt.handle().to_owned(), None)?;

    let interchain: InterchainEnv = starship.interchain_env();

    let juno = interchain.daemon(JUNO)?;
    let osmosis = interchain.daemon(OSMOSIS)?;

    // ### SETUP ###
    deploy_contracts(&juno, &osmosis)?;

    // ### CREATE THE CHANNEL BETWEEN THE 2 CLIENTS ###
    let client = IbcClient::new(IBC_CLIENT, juno.clone());
    let host = IbcHost::new("host", juno.clone());

    rt.block_on(
        InterchainChannelBuilder::default()
            .from_contracts(&client, &host)
            .create_channel(starship.client(), "simple-ica-v2"),
    )?;

    Ok(())
}

fn main() {
    dotenv().ok();
    use dotenv::dotenv;

    if let Err(ref err) = script() {
        log::error!("{}", err);
        err.chain()
            .skip(1)
            .for_each(|cause| log::error!("because: {}", cause));
        ::std::process::exit(1);
    }
}

fn deploy_contracts(juno: &Daemon, osmosis: &Daemon) -> anyhow::Result<()> {
    let juno_abstr = Abstract::deploy_on(juno.clone(), "1.0.0".parse().unwrap())?;

    // now deploy IBC stuff
    let client = IbcClient::new(IBC_CLIENT, juno.clone());
    let host = IbcHost::new("host", juno.clone());
    client.upload()?;
    host.upload()?;

    client.instantiate(
        &ibc_client::InstantiateMsg {
            ans_host_address: juno_abstr.ans_host.addr_str()?,
            chain: "juno-1".to_string(),
            version_control_address: juno_abstr.version_control.addr_str()?,
        },
        None,
        None,
    )?;

    let _osmo_abstr = Abstract::deploy_on(osmosis.clone(), "1.0.0".parse().unwrap())?;

    Ok(())
}
