use abstract_interface::Abstract;
use abstract_scripts::abstract_ibc::abstract_ibc_connection_with;
use anyhow::Result as AnyResult;
use cw_orch::prelude::*;
use cw_orch_polytone::Polytone;
use polytone::handshake::POLYTONE_VERSION;

pub fn ibc_abstract_setup<Chain: IbcQueryHandler, IBC: InterchainEnv<Chain>>(
    interchain: &IBC,
    origin_chain_id: &str,
    remote_chain_id: &str,
) -> AnyResult<(Abstract<Chain>, Abstract<Chain>)> {
    let origin_chain = interchain.chain(origin_chain_id).unwrap();
    let remote_chain = interchain.chain(remote_chain_id).unwrap();

    // Deploying abstract and the IBC abstract logic
    let abstr_origin =
        Abstract::deploy_on(origin_chain.clone(), origin_chain.sender().to_string())?;
    let abstr_remote =
        Abstract::deploy_on(remote_chain.clone(), remote_chain.sender().to_string())?;

    // Deploying polytone on both chains
    let origin_polytone = Polytone::deploy_on(origin_chain.clone(), None)?;
    let remote_polytone = Polytone::deploy_on(remote_chain.clone(), None)?;

    // Creating a connection between 2 polytone deployments
    interchain.create_contract_channel(
        &origin_polytone.note,
        &remote_polytone.voice,
        POLYTONE_VERSION,
        None, // Unordered channel
    )?;

    // Create the connection between client and host
    abstract_ibc_connection_with(&abstr_origin, interchain, &abstr_remote, &origin_polytone)?;

    Ok((abstr_origin, abstr_remote))
}

#[cfg(test)]
pub mod mock_test {
    use abstract_std::{
        ibc_client::QueryMsgFns, ibc_host::QueryMsgFns as _, objects::chain_name::ChainName,
    };

    use super::*;
    use crate::{JUNO, STARGAZE};

    /// This allows env_logger to start properly for tests
    /// The logs will be printed only if the test fails !
    pub fn logger_test_init() {
        let _ = env_logger::builder().is_test(true).try_init();
    }

    #[test]
    fn ibc_setup() -> AnyResult<()> {
        logger_test_init();
        let mock_interchain =
            MockBech32InterchainEnv::new(vec![(JUNO, "juno"), (STARGAZE, "stars")]);

        // We just verified all steps pass
        let (origin_abstr, remote_abstr) = ibc_abstract_setup(&mock_interchain, JUNO, STARGAZE)?;

        // We verify the host is active on the client on chain JUNO
        let remote_hosts = origin_abstr.ibc.client.list_remote_hosts()?;
        assert_eq!(remote_hosts.hosts.len(), 1);
        assert_eq!(remote_hosts.hosts[0].0, ChainName::from_chain_id(STARGAZE));

        // We verify the client is active on the host chain JUNO
        let remote_hosts = remote_abstr.ibc.host.client_proxies(None, None)?;
        assert_eq!(remote_hosts.chains.len(), 1);
        assert_eq!(remote_hosts.chains[0].0, ChainName::from_chain_id(JUNO));

        Ok(())
    }
}
