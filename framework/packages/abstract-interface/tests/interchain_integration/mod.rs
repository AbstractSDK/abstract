pub mod interchain_accounts;
pub mod module_to_module_interactions;

use abstract_interface::Abstract;
use cw_orch::anyhow;
use cw_orch::prelude::*;
use cw_orch_interchain::prelude::*;

pub const JUNO: &str = "juno-1";
pub const STARGAZE: &str = "stargaze-1";
pub const OSMOSIS: &str = "osmosis-1";

pub fn ibc_abstract_setup<Chain: IbcQueryHandler<Sender = Addr>, IBC: InterchainEnv<Chain>>(
    interchain: &IBC,
    origin_chain_id: &str,
    remote_chain_id: &str,
) -> anyhow::Result<(Abstract<Chain>, Abstract<Chain>)> {
    let origin_chain = interchain.get_chain(origin_chain_id).unwrap();
    let remote_chain = interchain.get_chain(remote_chain_id).unwrap();

    // Deploying abstract and the IBC abstract logic
    let abstr_origin = Abstract::deploy_on(origin_chain.clone(), ())?;
    let abstr_remote = Abstract::deploy_on(remote_chain.clone(), ())?;

    abstr_origin.connect_to(&abstr_remote, interchain)?;

    Ok((abstr_origin, abstr_remote))
}

/// This allows env_logger to start properly for tests
/// The logs will be printed only if the test fails !
pub fn logger_test_init() {
    let _ = env_logger::builder().is_test(true).try_init();
}
