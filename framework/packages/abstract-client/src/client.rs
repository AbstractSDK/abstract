use abstract_interface::{Abstract, AnsHost, VersionControl};
use cosmwasm_std::{Addr, BlockInfo, Coin, Uint128};
use cw_orch::{deploy::Deploy, environment::MutCwEnv, prelude::CwEnv};

use crate::{
    account::{Account, AccountBuilder},
    error::AbstractClientError,
    infrastructure::{Environment, Infrastructure},
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
        namespace: &str,
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
        namespace: &str,
    ) -> AbstractClientResult<Account<Chain>> {
        Account::from_namespace(&self.abstr, namespace)
    }

    pub fn sender(&self) -> Addr {
        self.environment().sender()
    }

    pub fn query_balance(
        &self,
        address: &Addr,
        denom: impl Into<String>,
    ) -> AbstractClientResult<Uint128> {
        let coins = self.balance(address, Some(denom.into()))?;
        // There will always be a single element in this case.
        Ok(coins[0].amount)
    }

    pub fn query_balances(&self, address: &Addr) -> AbstractClientResult<Vec<Coin>> {
        self.balance(address, None)
    }

    pub fn wait_blocks(&self, amount: u64) -> AbstractClientResult<()> {
        self.environment()
            .wait_blocks(amount)
            .map_err(Into::into)
            .map_err(Into::into)
    }

    pub fn wait_seconds(&self, amount: u64) -> AbstractClientResult<()> {
        self.environment()
            .wait_seconds(amount)
            .map_err(Into::into)
            .map_err(Into::into)
    }

    pub fn next_block(&self) -> AbstractClientResult<()> {
        self.environment()
            .next_block()
            .map_err(Into::into)
            .map_err(Into::into)
    }
}

impl<Chain: MutCwEnv> AbstractClient<Chain> {
    pub fn set_balance(&self, address: &Addr, amount: Vec<Coin>) -> AbstractClientResult<()> {
        self.environment()
            .set_balance(address, amount)
            .map_err(Into::into)
            .map_err(Into::into)
    }

    pub fn add_balance(&self, address: &Addr, amount: Vec<Coin>) -> AbstractClientResult<()> {
        self.environment()
            .add_balance(address, amount)
            .map_err(Into::into)
            .map_err(Into::into)
    }
}

#[cfg(feature = "test-utils")]
pub mod test_utils {
    use abstract_core::{
        self,
        objects::{
            pool_id::UncheckedPoolAddress, PoolMetadata, UncheckedChannelEntry,
            UncheckedContractEntry,
        },
    };
    use abstract_interface::{Abstract, ExecuteMsgFns};
    use cosmwasm_std::{Addr, Coin};
    use cw_asset::AssetInfoUnchecked;
    use cw_orch::{deploy::Deploy, prelude::Mock};

    use crate::infrastructure::Environment;

    use self::cw20_builder::Cw20Builder;

    use super::{AbstractClient, AbstractClientResult};

    impl AbstractClient<Mock> {
        pub fn builder(sender: impl Into<String>) -> AbstractClientBuilder {
            AbstractClientBuilder::new(sender.into())
        }

        pub fn cw20_builder(
            &self,
            name: impl Into<String>,
            symbol: impl Into<String>,
            decimals: u8,
        ) -> Cw20Builder {
            Cw20Builder::new(self.environment(), name.into(), symbol.into(), decimals)
        }
    }

    pub mod cw20_builder {
        // Re-exports to limit dependencies for consumer.
        pub use cw20::{msg::Cw20ExecuteMsgFns, *};
        pub use cw20_base::msg::{InstantiateMarketingInfo, QueryMsgFns as Cw20QueryMsgFns};
        pub use cw_plus_interface::cw20_base::Cw20Base;

        use cosmwasm_std::Addr;

        use cw_orch::prelude::{CwOrchInstantiate, CwOrchUpload, Mock};
        use cw_plus_interface::cw20_base::InstantiateMsg;

        use crate::client::AbstractClientResult;

        pub struct Cw20Builder {
            chain: Mock,
            name: String,
            symbol: String,
            decimals: u8,
            initial_balances: Vec<Cw20Coin>,
            mint: Option<MinterResponse>,
            marketing: Option<InstantiateMarketingInfo>,
            admin: Option<Addr>,
        }

        impl Cw20Builder {
            /// Creates a new [`Cw20Builder`]. Call [`crate::client::AbstractClient`] to create.
            pub(crate) fn new(chain: Mock, name: String, symbol: String, decimals: u8) -> Self {
                Self {
                    chain,
                    name,
                    symbol,
                    decimals,
                    initial_balances: vec![],
                    mint: None,
                    marketing: None,
                    admin: None,
                }
            }

            pub fn initial_balances(&mut self, initial_balances: Vec<Cw20Coin>) -> &mut Self {
                self.initial_balances = initial_balances;
                self
            }

            pub fn initial_balance(&mut self, initial_balance: Cw20Coin) -> &mut Self {
                self.initial_balances.push(initial_balance);
                self
            }

            pub fn mint(&mut self, mint: MinterResponse) -> &mut Self {
                self.mint = Some(mint);
                self
            }

            pub fn marketing(&mut self, marketing: InstantiateMarketingInfo) -> &mut Self {
                self.marketing = Some(marketing);
                self
            }

            pub fn admin(&mut self, admin: impl Into<String>) -> &mut Self {
                self.admin = Some(Addr::unchecked(admin.into()));
                self
            }

            pub fn instantiate_with_id(&self, id: &str) -> AbstractClientResult<Cw20Base<Mock>> {
                let cw20 = Cw20Base::new(id, self.chain.clone());

                // TODO: Consider adding error if the code-id is already uploaded. This would
                // imply that the user is trying to instantiate twice using the same id which would
                // overwrite the state.
                cw20.upload()?;
                cw20.instantiate(
                    &InstantiateMsg {
                        decimals: self.decimals,
                        mint: self.mint.clone(),
                        symbol: self.symbol.clone(),
                        name: self.name.clone(),
                        initial_balances: self.initial_balances.clone(),
                        marketing: self.marketing.clone(),
                    },
                    self.admin.as_ref(),
                    None,
                )?;
                Ok(cw20)
            }
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
        pub(crate) fn new(sender: impl Into<String>) -> Self {
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

        pub fn balances(&mut self, balances: Vec<(impl Into<String>, &[Coin])>) -> &mut Self {
            self.balances = balances
                .into_iter()
                .map(|b| (b.0.into(), b.1.to_vec()))
                .collect();
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
