//! # Abstract client Test utilities
//!
//! This module provides useful helpers for integration tests

use abstract_interface::{Abstract, ExecuteMsgFns};
use abstract_std::objects::{
    pool_id::UncheckedPoolAddress, PoolMetadata, UncheckedChannelEntry, UncheckedContractEntry,
};
use cw_asset::AssetInfoUnchecked;
use cw_orch::prelude::*;

use self::cw20_builder::Cw20Builder;
use crate::{
    client::{AbstractClient, AbstractClientResult},
    Environment,
};

impl<Chain: CwEnv> AbstractClient<Chain> {
    /// Abstract client builder
    pub fn builder(chain: Chain) -> AbstractClientBuilder<Chain> {
        AbstractClientBuilder::new(chain)
    }

    /// Cw20 contract builder
    pub fn cw20_builder(
        &self,
        name: impl Into<String>,
        symbol: impl Into<String>,
        decimals: u8,
    ) -> Cw20Builder<Chain> {
        Cw20Builder::new(self.environment(), name.into(), symbol.into(), decimals)
    }
}

/// A builder for setting up tests for `Abstract` in an environment where Abstract isn't deployed yet.
/// Example: [`Mock`](cw_orch::prelude::Mock) or a local [`Daemon`](cw_orch::prelude::Daemon).
pub struct AbstractClientBuilder<Chain: CwEnv> {
    pub(crate) chain: Chain,
    dexes: Vec<String>,
    contracts: Vec<(UncheckedContractEntry, String)>,
    pub(crate) assets: Vec<(String, AssetInfoUnchecked)>,
    channels: Vec<(UncheckedChannelEntry, String)>,
    pools: Vec<(UncheckedPoolAddress, PoolMetadata)>,
}

impl<Chain: CwEnv> AbstractClientBuilder<Chain> {
    pub(crate) fn new(chain: Chain) -> Self {
        Self {
            chain,
            dexes: vec![],
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

    /// Register dex on Abstract Name Service
    pub fn dex(&mut self, dex: &str) -> &mut Self {
        self.dexes.push(dex.to_string());
        self
    }

    /// Register dexes on Abstract Name Service
    pub fn dexes(&mut self, dexes: Vec<String>) -> &mut Self {
        self.dexes = dexes;
        self
    }

    /// Deploy abstract with current configuration
    pub fn build(&self) -> AbstractClientResult<AbstractClient<Chain>> {
        let abstr = Abstract::deploy_on(self.chain.clone(), self.chain.sender().into_string())?;
        self.update_ans(&abstr)?;

        AbstractClient::new(self.chain.clone())
    }

    fn update_ans(&self, abstr: &Abstract<Chain>) -> AbstractClientResult<()> {
        abstr.ans_host.update_dexes(self.dexes.clone(), vec![])?;
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
    use cosmwasm_std::Addr;
    pub use cw20::{msg::Cw20ExecuteMsgFns, *};
    pub use cw20_base::msg::{InstantiateMarketingInfo, QueryMsgFns as Cw20QueryMsgFns};
    use cw_orch::{
        environment::CwEnv,
        prelude::{CwOrchInstantiate, CwOrchUpload},
    };
    pub use cw_plus_interface::cw20_base::Cw20Base;
    use cw_plus_interface::cw20_base::InstantiateMsg;

    use crate::client::AbstractClientResult;

    /// A builder for creating and deploying `Cw20` contract in a [`CwEnv`](cw_orch::prelude::CwEnv) environment.
    pub struct Cw20Builder<Chain: CwEnv> {
        chain: Chain,
        name: String,
        symbol: String,
        decimals: u8,
        initial_balances: Vec<Cw20Coin>,
        mint: Option<MinterResponse>,
        marketing: Option<InstantiateMarketingInfo>,
        admin: Option<Addr>,
    }

    impl<Chain: CwEnv> Cw20Builder<Chain> {
        /// Creates a new [`Cw20Builder`]. Call [`crate::AbstractClient`] to create.
        pub(crate) fn new(chain: Chain, name: String, symbol: String, decimals: u8) -> Self {
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
        pub fn instantiate_with_id(&self, id: &str) -> AbstractClientResult<Cw20Base<Chain>> {
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
