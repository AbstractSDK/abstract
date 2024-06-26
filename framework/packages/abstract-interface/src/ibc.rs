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
        self.client.instantiate(
            &abstract_std::ibc_client::InstantiateMsg {
                ans_host_address: abstr.ans_host.addr_str()?,
                version_control_address: abstr.version_control.addr_str()?,
            },
            Some(admin),
            None,
        )?;

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
    use abstract_std::account_factory::ExecuteMsgFns;
    use abstract_std::ibc_client::ExecuteMsgFns as _;
    use abstract_std::ibc_client::QueryMsgFns;
    use abstract_std::ibc_host::ExecuteMsgFns as _;
    use abstract_std::objects::chain_name::ChainName;
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
            abstract_ibc_one_way_connection_with(self, remote_abstr, interchain)?;
            abstract_ibc_one_way_connection_with(remote_abstr, self, interchain)?;
            Ok(())
        }
    }

    pub fn abstract_ibc_one_way_connection_with<
        Chain: IbcQueryHandler,
        IBC: InterchainEnv<Chain>,
    >(
        abstr: &Abstract<Chain>,
        dest: &Abstract<Chain>,
        interchain: &IBC,
    ) -> Result<(), AbstractInterfaceError> {
        // First we register client and host respectively
        let chain1_id = abstr.ibc.client.get_chain().chain_id();
        let chain1_name = ChainName::from_chain_id(&chain1_id);

        let chain2_id = dest.ibc.client.get_chain().chain_id();
        let chain2_name = ChainName::from_chain_id(&chain2_id);

        // We get the polytone connection
        let polytone_connection =
            PolytoneConnection::deploy_between_if_needed(interchain, &chain1_id, &chain2_id)?;

        // First, we register the host with the client.
        // We register the polytone note with it because they are linked
        // This triggers an IBC message that is used to get back the proxy address
        let proxy_tx_result = abstr.ibc.client.register_infrastructure(
            chain2_name.clone(),
            dest.ibc.host.address()?.to_string(),
            polytone_connection.note.address()?.to_string(),
        )?;
        // We make sure the IBC execution is done so that the proxy address is saved inside the Abstract contract
        let _ = interchain.check_ibc(&chain1_id, proxy_tx_result)?;

        // Finally, we get the proxy address and register the proxy with the ibc host for the dest chain
        let proxy_address = abstr.ibc.client.host(chain2_name)?;

        dest.ibc
            .host
            .register_chain_proxy(chain1_name, proxy_address.remote_polytone_proxy.unwrap())?;

        dest.account_factory.update_config(
            None,
            Some(dest.ibc.host.address()?.to_string()),
            None,
            None,
        )?;

        Ok(())
    }
}
