#![allow(unused_imports)]
use abstract_client::AbstractClient;
use abstract_scripts::abstract_ibc::{
    has_abstract_ibc, has_polytone_connection, verify_abstract_ibc,
};
use abstract_scripts::NEUTRON_1;
use abstract_std::objects::chain_name::ChainName;
use abstract_std::objects::module::ModuleVersion;
use abstract_std::objects::namespace::Namespace;
use cw_orch::daemon::networks::neutron::NEUTRON_NETWORK;
use cw_orch::daemon::networks::{ARCHWAY_1, JUNO_1, OSMOSIS_1, PHOENIX_1};
use cw_orch::environment::ChainKind;
use cw_orch::prelude::*;
use cw_orch::tokio::runtime::Handle;
use cw_orch_interchain::prelude::*;
use tokio::runtime::Runtime;

fn main() -> cw_orch::anyhow::Result<()> {
    dotenv::dotenv()?;
    env_logger::init();

    let chains = vec![
        (JUNO_1, None),
        (PHOENIX_1, None),
        (ARCHWAY_1, None),
        (NEUTRON_1, None),
        (OSMOSIS_1, Some(std::env::var("OSMOSIS_MNEMONIC")?)),
    ];
    let runtime = Runtime::new()?;

    for src_chain in &chains {
        for dst_chain in &chains {
            if has_polytone_connection(src_chain.0.clone(), dst_chain.0.clone(), runtime.handle()) {
                if has_abstract_ibc(src_chain.0.clone(), dst_chain.0.clone(), runtime.handle()) {
                    // connect(src_chain.clone(), dst_chain.clone(), runtime.handle())?;
                } else {
                    println!(
                        "Abstract Not Ok for {}:{}:{}",
                        src_chain.0.chain_id,
                        dst_chain.0.chain_id,
                        verify_abstract_ibc(
                            src_chain.0.clone(),
                            dst_chain.0.clone(),
                            runtime.handle()
                        )
                        .unwrap_err()
                    );
                }
            } else {
                println!(
                    "Polytone Not Ok for {}:{}",
                    src_chain.0.chain_id, dst_chain.0.chain_id
                );
            }
        }
    }

    Ok(())
}

#[allow(dead_code)]
fn get_daemon(
    chain: ChainInfo,
    handle: &Handle,
    mnemonic: Option<String>,
    deployment_id: Option<String>,
) -> cw_orch::anyhow::Result<Daemon> {
    let mut builder = DaemonBuilder::default();
    builder.chain(chain).handle(handle);
    if let Some(mnemonic) = mnemonic {
        builder.mnemonic(mnemonic);
    }
    if let Some(deployment_id) = deployment_id {
        builder.deployment_id(deployment_id);
    }
    Ok(builder.build()?)
}

#[allow(dead_code)]
fn connect(
    (src_chain, src_mnemonic): (ChainInfo, Option<String>),
    (dst_chain, dst_mnemonic): (ChainInfo, Option<String>),
    handle: &Handle,
) -> cw_orch::anyhow::Result<()> {
    let src_daemon = get_daemon(src_chain.clone(), handle, src_mnemonic.clone(), None)?;
    let dst_daemon = get_daemon(dst_chain.clone(), handle, dst_mnemonic, None)?;

    let interchain = DaemonInterchainEnv::from_daemons(
        handle,
        vec![src_daemon.clone(), dst_daemon.clone()],
        &ChannelCreationValidator,
    );
    let client = AbstractClient::new(src_daemon)?;
    let remote_client = AbstractClient::new(dst_daemon)?;
    let account = client
        .account_builder()
        .namespace(Namespace::new("abstract")?)
        .fetch_if_namespace_claimed(true)
        .build()?;

    // We upgrade the local account. If it fails, it's ok (for instance if we're already updated)
    let _ = account.upgrade(ModuleVersion::Latest);

    // We install the ibc client on the account. If it fails, it's ok (for instance if we're already updated)
    let _ = account.set_ibc_status(true);

    // We create remote account
    let _ = account
        .remote_account_builder(&interchain, &remote_client)
        .build()?;
    Ok(())
}
