use abstract_client::GovernanceDetails;
use abstract_interface::Abstract;
use abstract_interface::AccountFactoryExecFns;
use cw_orch::daemon::senders::multiple_sender::MultiDaemon;
use cw_orch::{
    contract::Deploy,
    daemon::{networks::XION_TESTNET_1, Daemon},
    prelude::*,
};

pub fn main() -> cw_orch::anyhow::Result<()> {
    dotenv::dotenv()?;
    env_logger::init();
    let chain = MultiDaemon::builder().chain(XION_TESTNET_1).build()?;
    let abstr = Abstract::load_from(chain.clone())?;

    for i in 0..500 {
        for _ in 0..10 {
            abstr.account_factory.create_account(
                GovernanceDetails::Monarchy {
                    monarch: chain.sender().to_string(),
                },
                vec![],
                "Test-account",
                None,
                None,
                None,
                None,
                None,
                &[],
            )?;
        }

        println!("Sending Batch n° {i}");

        chain.rt_handle.block_on(chain.wallet().broadcast(None))?;
    }

    Ok(())
}
