use crate::{Abstract, AbstractInterfaceError, IbcClient, IbcHost, VersionControl};
use abstract_std::{IBC_CLIENT, IBC_HOST};
use cw_orch::prelude::*;
pub struct AbstractIbc<Chain: CwEnv> {
    pub client: IbcClient<Chain>,
    pub host: IbcHost<Chain>,
}

impl<Chain: CwEnv> AbstractIbc<Chain> {
    pub fn new(chain: &Chain) -> Self {
        let ibc_client = IbcClient::new(IBC_CLIENT, chain.clone());
        let ibc_host = IbcHost::new(IBC_HOST, chain.clone());
        Self {
            client: ibc_client,
            host: ibc_host,
        }
    }

    pub fn upload(&self) -> Result<(), crate::AbstractInterfaceError> {
        self.client.upload()?;
        self.host.upload()?;
        Ok(())
    }

    pub fn instantiate(&self, abstr: &Abstract<Chain>, admin: &Addr) -> Result<(), CwOrchError> {
        #[cfg(feature = "interchain")]
        let register_infrastructures =
            connection::list_ibc_infrastructures(self.host.environment().clone());

        self.client.instantiate(
            &abstract_std::ibc_client::InstantiateMsg {
                ans_host_address: abstr.ans_host.addr_str()?,
                version_control_address: abstr.version_control.addr_str()?,
            },
            Some(admin),
            None,
        )?;

        #[cfg(feature = "interchain")]
        for (chain, ibc_infrastructure) in register_infrastructures.counterparts {
            use abstract_std::ibc_client::ExecuteMsgFns;

            self.client.register_infrastructure(
                chain,
                ibc_infrastructure.remote_abstract_host,
                ibc_infrastructure.polytone_note,
            )?;
        }

        self.host.instantiate(
            &abstract_std::ibc_host::InstantiateMsg {
                ans_host_address: abstr.ans_host.addr_str()?,
                account_factory_address: abstr.account_factory.addr_str()?,
                version_control_address: abstr.version_control.addr_str()?,
            },
            Some(admin),
            None,
        )?;
        Ok(())
    }

    pub fn register(
        &self,
        version_control: &VersionControl<Chain>,
    ) -> Result<(), AbstractInterfaceError> {
        version_control.register_natives(vec![
            (
                self.client.as_instance(),
                ibc_client::contract::CONTRACT_VERSION.to_string(),
            ),
            (
                self.host.as_instance(),
                ibc_host::contract::CONTRACT_VERSION.to_string(),
            ),
        ])
    }
}

#[cfg(feature = "interchain")]
// Helpers to create connection with another chain
pub mod connection {
    use super::*;
    use abstract_std::ibc_client::{ExecuteMsgFns, QueryMsgFns};
    use abstract_std::ibc_host::ExecuteMsgFns as _;
    use abstract_std::objects::TruncatedChainId;
    use cw_orch::environment::Environment;
    use cw_orch_interchain::prelude::*;
    use cw_orch_polytone::interchain::PolytoneConnection;

    impl<Chain: IbcQueryHandler> Abstract<Chain> {
        /// This is used for creating a testing connection between two Abstract connections.
        ///
        /// If a polytone deployment is already , it uses the existing deployment, If it doesn't exist, it creates it
        ///
        /// You usually don't need this function on actual networks if you're not an Abstract maintainer
        pub fn connect_to<IBC: InterchainEnv<Chain>>(
            &self,
            remote_abstr: &Abstract<Chain>,
            interchain: &IBC,
        ) -> Result<(), AbstractInterfaceError> {
            connect_one_way_to(self, remote_abstr, interchain)?;
            connect_one_way_to(remote_abstr, self, interchain)?;
            Ok(())
        }
    }

    pub fn connect_one_way_to<Chain: IbcQueryHandler, IBC: InterchainEnv<Chain>>(
        abstr_client: &Abstract<Chain>,
        abstr_host: &Abstract<Chain>,
        interchain: &IBC,
    ) -> Result<(), AbstractInterfaceError> {
        // First we register client and host respectively
        let chain1_id = abstr_client.ibc.client.environment().chain_id();
        let chain1_name = TruncatedChainId::from_chain_id(&chain1_id);

        let chain2_id = abstr_host.ibc.client.environment().chain_id();
        let chain2_name = TruncatedChainId::from_chain_id(&chain2_id);

        // We get the polytone connection
        let polytone_connection =
            PolytoneConnection::deploy_between_if_needed(interchain, &chain1_id, &chain2_id)?;

        // First, we register the host with the client.
        // We register the polytone note with it because they are linked
        // This triggers an IBC message that is used to get back the proxy address
        let proxy_tx_result = abstr_client.ibc.client.register_infrastructure(
            chain2_name.clone(),
            abstr_host.ibc.host.address()?.to_string(),
            polytone_connection.note.address()?.to_string(),
        )?;
        // We make sure the IBC execution is done so that the proxy address is saved inside the Abstract contract
        interchain.await_and_check_packets(&chain1_id, proxy_tx_result)?;

        // Finally, we get the proxy address and register the proxy with the ibc host for the host chain
        let proxy_address = abstr_client.ibc.client.host(chain2_name)?;

        abstr_host
            .ibc
            .host
            .register_chain_proxy(chain1_name, proxy_address.remote_polytone_proxy.unwrap())?;

        Ok(())
    }

    pub fn list_ibc_infrastructures<Chain: CwEnv>(
        chain: Chain,
    ) -> abstract_std::ibc_client::ListIbcInfrastructureResponse {
        let Ok(polytone) = cw_orch_polytone::Polytone::load_from(chain) else {
            return abstract_std::ibc_client::ListIbcInfrastructureResponse {
                counterparts: vec![],
            };
        };
        let abstract_state = crate::AbstractDaemonState::default();
        let deployment_id = "default".to_owned();

        let mut counterparts = vec![];
        for connected_polytone in polytone.connected_polytones() {
            // TODO: It's crappy to rely on parse_network to get chain_name, but is there any other option?
            // Perhaps we should store daemon states in just `.chain_id`, instead of `.chain_name.chain_id`
            let Ok(chain_info) = networks::parse_network(&connected_polytone.chain_id) else {
                continue;
            };
            let chain_name = chain_info.network_info.chain_name.to_owned();
            let env_info = EnvironmentInfo {
                chain_id: connected_polytone.chain_id.clone(),
                chain_name,
                deployment_id: deployment_id.clone(),
            };
            if let Some(remote_abstract_host) = abstract_state.contract_addr(&env_info, IBC_HOST) {
                let truncated_chain_id =
                    abstract_std::objects::TruncatedChainId::from_chain_id(&env_info.chain_id);
                counterparts.push((
                    truncated_chain_id,
                    abstract_std::ibc_client::state::IbcInfrastructure {
                        polytone_note: connected_polytone.note,
                        remote_abstract_host: remote_abstract_host.into(),
                        remote_proxy: None,
                    },
                ))
            }
        }
        abstract_std::ibc_client::ListIbcInfrastructureResponse { counterparts }
    }
}
