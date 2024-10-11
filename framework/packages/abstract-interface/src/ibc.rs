use crate::{AbstractInterfaceError, IbcClient, IbcHost, Registry};
use abstract_std::{IBC_CLIENT, IBC_HOST};
use cw_orch::prelude::*;

#[derive(Clone)]
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

    pub fn instantiate(&self, admin: &Addr) -> Result<(), CwOrchError> {
        self.client.instantiate(
            &abstract_std::ibc_client::InstantiateMsg {},
            Some(admin),
            &[],
        )?;

        self.host
            .instantiate(&abstract_std::ibc_host::InstantiateMsg {}, Some(admin), &[])?;
        Ok(())
    }

    pub fn register(&self, registry: &Registry<Chain>) -> Result<(), AbstractInterfaceError> {
        registry.register_natives(vec![
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

    pub fn call_as(&self, sender: &<Chain as TxHandler>::Sender) -> Self {
        Self {
            client: self.client.call_as(sender),
            host: self.host.call_as(sender),
        }
    }
}

#[cfg(feature = "interchain")]
// Helpers to create connection with another chain
pub mod connection {
    use super::*;
    use crate::Abstract;
    use abstract_std::ibc_client::ExecuteMsgFns as _;
    use abstract_std::ibc_client::QueryMsgFns;
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
}
