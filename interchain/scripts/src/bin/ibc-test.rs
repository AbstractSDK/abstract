use abstract_interface::ManagerExecFns;
use abstract_interface::{Abstract, AbstractAccount};
use abstract_scripts::abstract_ibc::abstract_ibc_connection_with;
use abstract_scripts::{NEUTRON_1, ROLLKIT_TESTNET};
use abstract_std::ibc_host::HostAction;
use abstract_std::objects::chain_name::ChainName;
use abstract_std::objects::AccountId;
use abstract_std::{ibc_client, proxy, PROXY};
use cosmwasm_std::to_json_binary;
use cw_orch::daemon::networks::{ARCHWAY_1, HARPOON_4, JUNO_1, OSMO_5, PHOENIX_1, PION_1};
use cw_orch::prelude::*;
use cw_orch::tokio::runtime::Handle;
use cw_orch_polytone::Polytone;
use tokio::runtime::Runtime;

/// Connect IBC between two chains.
/// @TODO update this to take in the networks as arguments.
fn main() -> cw_orch::anyhow::Result<()> {
    dotenv::dotenv()?;
    env_logger::init();

    let chains = vec![
        (HARPOON_4, None),
        (PION_1, None),
        // (OSMOSIS_1, Some(std::env::var("OSMOSIS_MNEMONIC")?)),
    ];
    let runtime = Runtime::new()?;

    let src_chain = &chains[1];
    let dst_chain = &chains[0];

    ibc_test(src_chain.clone(), dst_chain.clone(), runtime.handle())?;

    Ok(())
}

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

pub fn get_deployment_id(src_chain: &ChainInfo, dst_chain: &ChainInfo) -> String {
    format!("{}-->{}", src_chain.chain_id, dst_chain.chain_id)
}

fn ibc_test(
    (src_chain, src_mnemonic): (ChainInfo, Option<String>),
    (dst_chain, dst_mnemonic): (ChainInfo, Option<String>),
    handle: &Handle,
) -> cw_orch::anyhow::Result<()> {
    let src_daemon = get_daemon(src_chain.clone(), handle, src_mnemonic.clone(), None)?;
    let dst_daemon = get_daemon(dst_chain.clone(), handle, dst_mnemonic, None)?;

    let src_abstract = Abstract::load_from(src_daemon.clone())?;
    let dst_abstract = Abstract::load_from(dst_daemon.clone())?;

    let interchain = DaemonInterchainEnv::from_daemons(
        handle,
        vec![src_daemon, dst_daemon],
        &ChannelCreationValidator,
    );
    let account = AbstractAccount::new(&src_abstract, AccountId::local(0));

    let tx_response = account.manager.exec_on_module(
        to_json_binary(&proxy::ExecuteMsg::IbcAction {
            msg: ibc_client::ExecuteMsg::RemoteAction {
                host_chain: ChainName::from_chain_id(dst_chain.chain_id).to_string(),
                action: HostAction::Dispatch {
                    manager_msgs: vec![],
                },
            },
        })?,
        PROXY.to_string(),
        &[],
    )?;

    interchain.wait_ibc(src_chain.chain_id, tx_response)?;

    Ok(())
}
