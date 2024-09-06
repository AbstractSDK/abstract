use abstract_std::{
    ans_host::state::{ASSET_ADDRESSES, CHANNELS, CONTRACT_ADDRESSES},
    objects::{
        account::ACCOUNT_ID,
        common_namespace::{ADMIN_NAMESPACE, OWNERSHIP_STORAGE_KEY},
        gov_type::GovernanceDetails,
        ownership::Ownership,
        AccountId, AssetEntry, ChannelEntry, ContractEntry,
    },
    version_control::{state::ACCOUNT_ADDRESSES, Account},
};
use cosmwasm_std::{testing::MockApi, Addr};
use cw_asset::AssetInfo;
use cw_storage_plus::Item;

use crate::prelude::*;

pub struct AbstractMockQuerierBuilder {
    builder: MockQuerierBuilder,
    abstract_addrs: AbstractMockAddrs,
}

// ANCHOR: account
impl AbstractMockQuerierBuilder {
    pub fn new(mock_api: MockApi) -> AbstractMockQuerierBuilder {
        AbstractMockQuerierBuilder {
            builder: MockQuerierBuilder::default(),
            abstract_addrs: AbstractMockAddrs::new(mock_api),
        }
    }

    /// Mock the existence of an Account by setting the Account id for the proxy and manager along with registering the account to version control.
    pub fn account(mut self, account_base: &Account, account_id: AccountId) -> Self {
        self.builder = self
            .builder
            .with_contract_item(&account_base.proxy, ACCOUNT_ID, &account_id)
            .with_contract_item(&account_base.manager, ACCOUNT_ID, &account_id)
            .with_contract_item(
                &account_base.proxy,
                Item::new(ADMIN_NAMESPACE),
                &Some(account_base.manager.clone()),
            )
            // Setup the account owner as the test owner
            .with_contract_item(
                &account_base.manager,
                Item::new(OWNERSHIP_STORAGE_KEY),
                &Some(Ownership {
                    owner: GovernanceDetails::Monarchy {
                        monarch: self.abstract_addrs.owner.clone(),
                    },
                    pending_owner: None,
                    pending_expiry: None,
                }),
            )
            .with_contract_map_entry(
                &self.abstract_addrs.version_control,
                ACCOUNT_ADDRESSES,
                (&account_id, account_base.clone()),
            );

        self
    }
    // ANCHOR_END: account

    /// Add mock assets into ANS
    pub fn assets(mut self, assets: Vec<(&AssetEntry, AssetInfo)>) -> Self {
        self.builder = self.builder.with_contract_map_entries(
            &self.abstract_addrs.ans_host,
            ASSET_ADDRESSES,
            assets,
        );

        self
    }

    pub fn contracts(mut self, contracts: Vec<(&ContractEntry, Addr)>) -> Self {
        self.builder = self.builder.with_contract_map_entries(
            &self.abstract_addrs.ans_host,
            CONTRACT_ADDRESSES,
            contracts,
        );

        self
    }

    pub fn channels(mut self, channels: Vec<(&ChannelEntry, String)>) -> Self {
        self.builder = self.builder.with_contract_map_entries(
            &self.abstract_addrs.ans_host,
            CHANNELS,
            channels,
        );

        self
    }

    /// Change the version control address. Any account added after this will be registered to this address.
    pub fn set_version_control(mut self, version_control: Addr) -> Self {
        self.abstract_addrs.version_control = version_control;
        self
    }

    pub fn ans(self, _ans: MockAnsHost) -> Self {
        self
    }

    pub fn builder(self) -> MockQuerierBuilder {
        self.builder
    }

    pub fn addrs(&self) -> &AbstractMockAddrs {
        &self.abstract_addrs
    }

    pub fn build(self) -> MockQuerier {
        self.builder.build()
    }
}
