use abstract_interface::{Abstract, AnsHost, VersionControl};
use cosmwasm_std::BlockInfo;
use cw_orch::{deploy::Deploy, prelude::CwEnv};

use crate::{
    account::{Account, AccountBuilder},
    error::AbstractClientError,
    infrastructure::Infrastructure,
    publisher::{Publisher, PublisherBuilder},
};

pub struct AbstractClient<Chain: CwEnv> {
    pub(crate) abstr: Abstract<Chain>,
}

pub type AbstractClientResult<T> = Result<T, AbstractClientError>;

impl<Chain: CwEnv> AbstractClient<Chain> {
    pub fn new(chain: Chain) -> AbstractClientResult<Self> {
        let abstr = Abstract::load_from(chain)?;
        Ok(Self { abstr })
    }

    pub fn name_service(&self) -> &AnsHost<Chain> {
        &self.abstr.ans_host
    }

    pub fn version_control(&self) -> &VersionControl<Chain> {
        &self.abstr.version_control
    }

    pub fn block_info(&self) -> AbstractClientResult<BlockInfo> {
        self.environment()
            .block_info()
            .map_err(Into::<cw_orch::prelude::CwOrchError>::into)
            .map_err(Into::<AbstractClientError>::into)
    }

    pub fn get_publisher_from_namespace(
        &self,
        namespace: String,
    ) -> AbstractClientResult<Publisher<Chain>> {
        Ok(Publisher::new(self.get_account_from_namespace(namespace)?))
    }

    pub fn publisher_builder(&self) -> PublisherBuilder<Chain> {
        PublisherBuilder::new(AccountBuilder::new(&self.abstr))
    }

    pub fn account_builder(&self) -> AccountBuilder<Chain> {
        AccountBuilder::new(&self.abstr)
    }

    pub fn get_account_from_namespace(
        &self,
        namespace: String,
    ) -> AbstractClientResult<Account<Chain>> {
        Account::from_namespace(&self.abstr, namespace)
    }
}

#[cfg(feature = "test-utils")]
mod test_utils {
    use abstract_core::{
        self,
        objects::{
            pool_id::UncheckedPoolAddress, PoolMetadata, UncheckedChannelEntry,
            UncheckedContractEntry,
        },
    };
    use abstract_interface::{Abstract, ExecuteMsgFns};
    use cosmwasm_std::{Addr, Coin, Uint128};
    use cw_asset::AssetInfoUnchecked;
    use cw_orch::{
        deploy::Deploy,
        prelude::{Mock, TxHandler},
    };

    use crate::infrastructure::Infrastructure;

    use super::{AbstractClient, AbstractClientResult};

    impl AbstractClient<Mock> {
        pub fn builder(sender: impl Into<String>) -> AbstractClientBuilder {
            AbstractClientBuilder::new(sender.into())
        }

        pub fn wait_blocks(&self, amount: u64) -> AbstractClientResult<()> {
            self.environment().wait_blocks(amount).map_err(Into::into)
        }

        // TODO: Also have this in non `Mock` case
        pub fn query_balance(&self, address: &Addr, denom: &str) -> AbstractClientResult<Uint128> {
            self.environment()
                .query_balance(address, denom)
                .map_err(Into::into)
        }
    }

    pub struct AbstractClientBuilder {
        mock: Mock,
        sender: String,
        balances: Vec<(String, Vec<Coin>)>,
        contracts: Vec<(UncheckedContractEntry, String)>,
        assets: Vec<(String, AssetInfoUnchecked)>,
        channels: Vec<(UncheckedChannelEntry, String)>,
        pools: Vec<(UncheckedPoolAddress, PoolMetadata)>,
    }

    impl AbstractClientBuilder {
        pub fn new(sender: impl Into<String>) -> Self {
            let sender: String = sender.into();
            Self {
                mock: Mock::new(&Addr::unchecked(&sender)),
                sender,
                balances: vec![],
                contracts: vec![],
                assets: vec![],
                channels: vec![],
                pools: vec![],
            }
        }

        pub fn contract(
            &mut self,
            contract_entry: UncheckedContractEntry,
            address: impl Into<String>,
        ) -> &mut Self {
            self.contracts.push((contract_entry, address.into()));
            self
        }

        pub fn contracts(&mut self, contracts: Vec<(UncheckedContractEntry, String)>) -> &mut Self {
            self.contracts = contracts;
            self
        }

        pub fn asset(
            &mut self,
            name: impl Into<String>,
            asset_info: AssetInfoUnchecked,
        ) -> &mut Self {
            self.assets.push((name.into(), asset_info));
            self
        }

        pub fn assets(&mut self, assets: Vec<(String, AssetInfoUnchecked)>) -> &mut Self {
            self.assets = assets;
            self
        }

        pub fn channel(
            &mut self,
            channel_entry: UncheckedChannelEntry,
            channel_value: impl Into<String>,
        ) -> &mut Self {
            self.channels.push((channel_entry, channel_value.into()));
            self
        }

        pub fn channels(&mut self, channels: Vec<(UncheckedChannelEntry, String)>) -> &mut Self {
            self.channels = channels;
            self
        }

        pub fn pool(
            &mut self,
            pool_address: UncheckedPoolAddress,
            pool_metadata: PoolMetadata,
        ) -> &mut Self {
            self.pools.push((pool_address, pool_metadata));
            self
        }

        pub fn pools(&mut self, pools: Vec<(UncheckedPoolAddress, PoolMetadata)>) -> &mut Self {
            self.pools = pools;
            self
        }

        pub fn balance(&mut self, address: impl Into<String>, amount: Vec<Coin>) -> &mut Self {
            self.balances.push((address.into(), amount));
            self
        }

        pub fn balances(&mut self, balances: Vec<(String, Vec<Coin>)>) -> &mut Self {
            self.balances = balances;
            self
        }

        pub fn build(&self) -> AbstractClientResult<AbstractClient<Mock>> {
            let abstr = Abstract::deploy_on(self.mock.clone(), self.sender.clone())?;
            self.update_ans(&abstr)?;
            self.update_balances()?;

            AbstractClient::new(self.mock.clone())
        }

        fn update_balances(&self) -> AbstractClientResult<()> {
            self.balances
                .iter()
                .try_for_each(|(address, amount)| -> AbstractClientResult<()> {
                    self.mock
                        .set_balance(&Addr::unchecked(address), amount.to_vec())?;
                    Ok(())
                })?;
            Ok(())
        }

        fn update_ans(&self, abstr: &Abstract<Mock>) -> AbstractClientResult<()> {
            abstr
                .ans_host
                .update_contract_addresses(self.contracts.clone(), vec![])?;
            abstr
                .ans_host
                .update_asset_addresses(self.assets.clone(), vec![])?;
            abstr
                .ans_host
                .update_channels(self.channels.clone(), vec![])?;
            abstr.ans_host.update_pools(self.pools.clone(), vec![])?;

            Ok(())
        }
    }
}
