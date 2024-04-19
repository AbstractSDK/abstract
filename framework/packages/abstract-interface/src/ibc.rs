use abstract_std::{IBC_CLIENT, IBC_HOST};
use cw_orch::prelude::*;

use crate::{Abstract, AbstractInterfaceError, IbcClient, IbcHost, VersionControl};

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
