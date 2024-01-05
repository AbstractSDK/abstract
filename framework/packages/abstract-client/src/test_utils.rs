//! # Abstract client Test utilities
//!
//! This module provides useful helpers for integration tests

use abstract_core::{
    self,
    objects::{
        pool_id::UncheckedPoolAddress, PoolMetadata, UncheckedChannelEntry, UncheckedContractEntry,
    },
};
use abstract_interface::{Abstract, ExecuteMsgFns};
use cosmwasm_std::{Addr, Coin};
use cw_asset::AssetInfoUnchecked;
use cw_orch::{deploy::Deploy, prelude::Mock};

use crate::{
    client::{AbstractClient, AbstractClientResult},
    infrastructure::Environment,
};

use self::cw20_builder::Cw20Builder;

impl AbstractClient<Mock> {
    /// Abstract client builder
    pub fn builder(sender: impl Into<String>) -> AbstractClientBuilder {
        AbstractClientBuilder::new(sender.into())
    }

    /// Cw20 contract builder
    pub fn cw20_builder(
        &self,
        name: impl Into<String>,
        symbol: impl Into<String>,
        decimals: u8,
    ) -> Cw20Builder {
        Cw20Builder::new(self.environment(), name.into(), symbol.into(), decimals)
    }
}

/// A builder for setting up tests for `Abstract` in a [`Mock`] environment.
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

    /// Register contract on Abstract Name Service
    pub fn contract(
        &mut self,
        contract_entry: UncheckedContractEntry,
        address: impl Into<String>,
    ) -> &mut Self {
        self.contracts.push((contract_entry, address.into()));
        self
    }

    /// Register contracts on Abstract Name Service
    pub fn contracts(&mut self, contracts: Vec<(UncheckedContractEntry, String)>) -> &mut Self {
        self.contracts = contracts;
        self
    }

    /// Register asset on Abstract Name Service
    pub fn asset(&mut self, name: impl Into<String>, asset_info: AssetInfoUnchecked) -> &mut Self {
        self.assets.push((name.into(), asset_info));
        self
    }

    /// Register assets on Abstract Name Service
    pub fn assets(&mut self, assets: Vec<(String, AssetInfoUnchecked)>) -> &mut Self {
        self.assets = assets;
        self
    }

    /// Register ibc channel on Abstract Name Service
    pub fn channel(
        &mut self,
        channel_entry: UncheckedChannelEntry,
        channel_value: impl Into<String>,
    ) -> &mut Self {
        self.channels.push((channel_entry, channel_value.into()));
        self
    }

    /// Register ibc channels on Abstract Name Service
    pub fn channels(&mut self, channels: Vec<(UncheckedChannelEntry, String)>) -> &mut Self {
        self.channels = channels;
        self
    }

    /// Register liquidity pool on Abstract Name Service
    pub fn pool(
        &mut self,
        pool_address: UncheckedPoolAddress,
        pool_metadata: PoolMetadata,
    ) -> &mut Self {
        self.pools.push((pool_address, pool_metadata));
        self
    }

    /// Register liquidity pools on Abstract Name Service
    pub fn pools(&mut self, pools: Vec<(UncheckedPoolAddress, PoolMetadata)>) -> &mut Self {
        self.pools = pools;
        self
    }

    /// Set on chain balance of address
    pub fn balance(&mut self, address: impl Into<String>, amount: Vec<Coin>) -> &mut Self {
        self.balances.push((address.into(), amount));
        self
    }

    /// Set on chain balances of addresses
    pub fn balances(&mut self, balances: Vec<(impl Into<String>, &[Coin])>) -> &mut Self {
        self.balances = balances
            .into_iter()
            .map(|b| (b.0.into(), b.1.to_vec()))
            .collect();
        self
    }

    /// Deploy abstract with current configuration
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

pub mod cw20_builder {
    //! # CW20 Builder

    // Re-exports to limit dependencies for consumer.
    pub use cw20::{msg::Cw20ExecuteMsgFns, *};
    pub use cw20_base::msg::{InstantiateMarketingInfo, QueryMsgFns as Cw20QueryMsgFns};
    pub use cw_plus_interface::cw20_base::Cw20Base;

    use cosmwasm_std::Addr;

    use cw_orch::prelude::{CwOrchInstantiate, CwOrchUpload, Mock};
    use cw_plus_interface::cw20_base::InstantiateMsg;

    use crate::client::AbstractClientResult;

    /// A builder for creating and deploying `Cw20` contract in a [`Mock`] environment.
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

        /// Set initial cw20 balance
        pub fn initial_balance(&mut self, initial_balance: Cw20Coin) -> &mut Self {
            self.initial_balances.push(initial_balance);
            self
        }

        /// Set initial cw20 balances
        pub fn initial_balances(&mut self, initial_balances: Vec<Cw20Coin>) -> &mut Self {
            self.initial_balances = initial_balances;
            self
        }

        /// Set minter
        pub fn mint(&mut self, mint: MinterResponse) -> &mut Self {
            self.mint = Some(mint);
            self
        }

        /// Set marketing info
        pub fn marketing(&mut self, marketing: InstantiateMarketingInfo) -> &mut Self {
            self.marketing = Some(marketing);
            self
        }

        /// Set admin
        pub fn admin(&mut self, admin: impl Into<String>) -> &mut Self {
            self.admin = Some(Addr::unchecked(admin.into()));
            self
        }

        /// Instantiate with provided module id
        // TODO: we can rename it to `build()` as other methods and take {module-id}-{symbol} as id instead
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
