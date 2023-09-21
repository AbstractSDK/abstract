use abstract_interface::Abstract;
use abstract_interface_scripts::abstract_ibc::abstract_ibc_connection_with;
use anyhow::Result as AnyResult;

use cw_orch::deploy::Deploy;
use cw_orch::prelude::*;
use cw_orch_polytone::{Polytone, PolytoneConnection};
use tokio::runtime::Runtime;

pub fn ibc_abstract_setup<Chain: IbcQueryHandler, IBC: InterchainEnv<Chain>>(
    rt: &Runtime,
    interchain: &IBC,
    chain_id_1: &str,
    chain_id_2: &str,
) -> AnyResult<(Abstract<Chain>, Abstract<Chain>)> {
    let chain1 = interchain.chain(chain_id_1).unwrap();
    let chain2 = interchain.chain(chain_id_2).unwrap();

    // Deploying abstract and the IBC abstract logic
    let abstr_1 = Abstract::deploy_on(chain1.clone(), chain1.sender().to_string())?;
    let abstr_2 = Abstract::deploy_on(chain2.clone(), chain2.sender().to_string())?;

    // Deploying polytone on both chains
    let polytone_1 = Polytone::deploy_on(chain1.clone(), None)?;
    let polytone_2 = Polytone::deploy_on(chain2.clone(), None)?;

    // Creating a connection between 2 polytone deployments
    let polytone_connection = rt.block_on(PolytoneConnection::connect(
        interchain,
        &polytone_1,
        &polytone_2,
    ))?;

    // Create the connection between client and host
    abstract_ibc_connection_with(&abstr_1, rt, interchain, &abstr_2, &polytone_connection)?;

    Ok((abstr_1, abstr_2))
}

#[cfg(test)]
pub mod mock_test {
    use abstract_core::{
        ibc_client::QueryMsgFns, ibc_host::QueryMsgFns as _, objects::chain_name::ChainName,
    };
    use cosmwasm_std::Addr;

    use crate::{JUNO, STARGAZE};

    use super::*;

    /// This allows env_logger to start properly for tests
    /// The logs will be printed only if the test fails !
    pub fn logger_test_init() {
        let _ = env_logger::builder().is_test(true).try_init();
    }

    #[test]
    fn ibc_setup() -> AnyResult<()> {
        logger_test_init();

        let rt = Runtime::new()?;
        let sender = Addr::unchecked("sender");
        let mock_interchain = MockInterchainEnv::new(vec![(JUNO, &sender), (STARGAZE, &sender)]);

        // We just verified all steps pass
        let (abstr1, abstr2) = ibc_abstract_setup(&rt, &mock_interchain, JUNO, STARGAZE)?;

        // We verify the host is active on the client on chain JUNO
        let remote_hosts = abstr1.ibc.client.list_remote_hosts()?;
        assert_eq!(remote_hosts.hosts.len(), 1);
        assert_eq!(remote_hosts.hosts[0].0, ChainName::from_chain_id(STARGAZE));

        // We verify the client is active on the host chain JUNO
        let remote_hosts = abstr2.ibc.host.registered_chains(None, None)?;
        assert_eq!(remote_hosts.chains.len(), 1);
        assert_eq!(remote_hosts.chains[0].0, ChainName::from_chain_id(JUNO));

        Ok(())
    }
}
