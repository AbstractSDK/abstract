//! # Abstract client Test utilities
//!
//! This module provides useful helpers for integration tests

use std::collections::HashMap;

use abstract_interface::{Abstract, ExecuteMsgFns};
use abstract_std::objects::{
    pool_id::UncheckedPoolAddress, PoolMetadata, UncheckedChannelEntry, UncheckedContractEntry,
};
use cw_asset::AssetInfoUnchecked;
use cw_orch::prelude::*;
use cw_orch_interchain::{IbcQueryHandler, InterchainEnv};

use crate::{
    client::{AbstractClient, AbstractClientResult},
    AbstractClientBuilder, AbstractClientError, Environment,
};

use super::client::AbstractInterchainClient;

impl<Chain: IbcQueryHandler> AbstractInterchainClient<Chain> {
    /// Abstract client builder
    pub fn builder() -> AbstractInterchainClientBuilder<Chain> {
        AbstractInterchainClientBuilder::new()
    }
}

/// A builder for setting up tests for `Abstract` in an environment where Abstract isn't deployed yet.
/// Example: [`Mock`](cw_orch::prelude::Mock) or a local [`Daemon`](cw_orch::prelude::Daemon).
pub struct AbstractInterchainClientBuilder<Chain: IbcQueryHandler> {
    builders: Vec<AbstractClientBuilder<Chain>>,
    post_setup_fn: Option<Box<dyn Fn(&AbstractClient<Chain>) -> AbstractClientResult<()>>>,
}
impl<Chain: IbcQueryHandler> Default for AbstractInterchainClientBuilder<Chain> {
    fn default() -> Self {
        Self {
            builders: Default::default(),
            post_setup_fn: Default::default(),
        }
    }
}

impl<Chain: IbcQueryHandler> AbstractInterchainClientBuilder<Chain> {
    pub(crate) fn new() -> Self {
        Self::default()
    }

    pub fn chain(&mut self, chain: Chain) -> &mut Self {
        self.builders.push(AbstractClientBuilder::new(chain));
        self
    }

    /// Register contract on Abstract Name Service
    pub fn contract(
        &mut self,
        contract_entry: UncheckedContractEntry,
        address: impl Into<String>,
    ) -> &mut Self {
        self.builders
            .last_mut()
            .unwrap()
            .contract(contract_entry, address);
        self
    }

    /// Register contracts on Abstract Name Service
    pub fn contracts(&mut self, contracts: Vec<(UncheckedContractEntry, String)>) -> &mut Self {
        self.builders.last_mut().unwrap().contracts(contracts);
        self
    }

    /// Register asset on Abstract Name Service
    pub fn asset(&mut self, name: impl Into<String>, asset_info: AssetInfoUnchecked) -> &mut Self {
        self.builders.last_mut().unwrap().asset(name, asset_info);
        self
    }

    /// Register assets on Abstract Name Service
    pub fn assets(&mut self, assets: Vec<(String, AssetInfoUnchecked)>) -> &mut Self {
        self.builders.last_mut().unwrap().assets(assets);
        self
    }

    /// Register ibc channel on Abstract Name Service
    pub fn channel(
        &mut self,
        channel_entry: UncheckedChannelEntry,
        channel_value: impl Into<String>,
    ) -> &mut Self {
        self.builders
            .last_mut()
            .unwrap()
            .channel(channel_entry, channel_value);
        self
    }

    /// Register ibc channels on Abstract Name Service
    pub fn channels(&mut self, channels: Vec<(UncheckedChannelEntry, String)>) -> &mut Self {
        self.builders.last_mut().unwrap().channels(channels);
        self
    }

    /// Register liquidity pool on Abstract Name Service
    pub fn pool(
        &mut self,
        pool_address: UncheckedPoolAddress,
        pool_metadata: PoolMetadata,
    ) -> &mut Self {
        self.builders
            .last_mut()
            .unwrap()
            .pool(pool_address, pool_metadata);
        self
    }

    /// Register liquidity pools on Abstract Name Service
    pub fn pools(&mut self, pools: Vec<(UncheckedPoolAddress, PoolMetadata)>) -> &mut Self {
        self.builders.last_mut().unwrap().pools(pools);
        self
    }

    /// Register dex on Abstract Name Service
    pub fn dex(&mut self, dex: &str) -> &mut Self {
        self.builders.last_mut().unwrap().dex(dex);
        self
    }

    /// Register dexes on Abstract Name Service
    pub fn dexes(&mut self, dexes: Vec<String>) -> &mut Self {
        self.builders.last_mut().unwrap().dexes(dexes);
        self
    }

    /// Deploy abstract with current configuration
    pub fn build<IBC: InterchainEnv<Chain>>(
        &self,
        interchain: &IBC,
    ) -> AbstractClientResult<AbstractInterchainClient<Chain>> {
        // First we propagate the different informations cross-chain
        // Propagating Assets

        todo!();

        // Propagating Dexes
        todo!();

        // Then we build all clients
        let all_abstrs: HashMap<_, _> = self
            .builders
            .iter()
            .map(|b| Ok::<_, AbstractClientError>((b.chain.chain_id(), b.build()?)))
            .collect::<Result<_, _>>()?;

        // Then we connect all clients
        let all_chains: Vec<_> = all_abstrs.keys().collect();
        // We use this loop pattern because `ibc_connection_with` connects chains in both ways
        for i in 0..all_chains.len() {
            for j in (i + 1)..all_chains.len() {
                all_abstrs
                    .get(all_chains[i])
                    .unwrap()
                    .connect_to(all_abstrs.get(all_chains[j]).unwrap(), interchain)?;
            }
        }

        if let Some(function) = self.post_setup_fn {
            all_abstrs
                .iter()
                .try_for_each(|(_, abstr)| function(abstr))?;
        }

        // Finally we return
        Ok(AbstractInterchainClient {
            abstracts: all_abstrs,
        })
    }

    pub fn post_setup_function(
        &mut self,
        function: impl Fn(&AbstractClient<Chain>) -> AbstractClientResult<()> + 'static,
    ) -> &mut Self {
        self.post_setup_fn = Some(Box::new(function));
        self
    }
}

impl AbstractInterchainClientBuilder<Daemon> {
    /// Deploy abstract with current configuration
    pub fn build_daemon<IBC: InterchainEnv<Daemon>>(
        &mut self,
        interchain: &IBC,
    ) -> AbstractClientResult<AbstractInterchainClient<Daemon>> {
        // First we propagate the different informations cross-chain
        // Propagating Assets
        let chain_len = self.builders.len();

        for i in 0..chain_len {
            for j in 0..chain_len {
                if i != j {
                    let asset_chain = &self.builders[j].chain.clone();
                    for (asset_name, asset) in self.builders[j].assets.clone() {
                        // We check the availability of an asset that is registered on chain j, on chain i (j = osmosis, i = archway for instance)
                        // We check all the channels to make sure the asset exists on this channel

                        let asset_denom = match asset {
                            cw_asset::AssetInfoBase::Native(denom) => denom,
                            cw_asset::AssetInfoBase::Cw20(address) => address,
                            _ => unimplemented!(),
                        };
                        let ibc: cw_orch::daemon::queriers::Ibc = self.builders[i].chain.querier();
                        // We look for a suitable channel for our ics20 token
                        for (channel, channel_id) in self.builders[i].channels.clone() {
                            if channel.connected_chain != asset_chain.chain_id()
                                || channel.protocol != "ics20-1"
                            {
                                continue;
                            }
                            let trace = format!(
                                "{}/{}/{}",
                                asset_chain.chain_id(),
                                channel_id,
                                asset_denom
                            );
                            // TODO, can this fail ?
                            let denom_hash = asset_chain
                                .rt_handle
                                .block_on(ibc._denom_hash(trace))
                                .unwrap();

                            // If we find a denom hash that works, we register it inside the i builder
                            self.builders[i].asset(
                                asset_name.clone(),
                                cw_asset::AssetInfoBase::Native(format!("ibc/{}", denom_hash)),
                            );
                        }
                    }
                }
            }
        }

        todo!();

        // Propagating Dexes
        todo!();

        // Then we build all clients
        let all_abstrs: HashMap<_, _> = self
            .builders
            .iter()
            .map(|b| Ok::<_, AbstractClientError>((b.chain.chain_id(), b.build()?)))
            .collect::<Result<_, _>>()?;

        // Then we connect all clients
        let all_chains: Vec<_> = all_abstrs.keys().collect();
        // We use this loop pattern because `ibc_connection_with` connects chains in both ways
        for i in 0..all_chains.len() {
            for j in (i + 1)..all_chains.len() {
                all_abstrs
                    .get(all_chains[i])
                    .unwrap()
                    .connect_to(all_abstrs.get(all_chains[j]).unwrap(), interchain)?;
            }
        }

        if let Some(function) = self.post_setup_fn {
            all_abstrs
                .iter()
                .try_for_each(|(_, abstr)| function(abstr))?;
        }

        // Finally we return
        Ok(AbstractInterchainClient {
            abstracts: all_abstrs,
        })
    }
}
