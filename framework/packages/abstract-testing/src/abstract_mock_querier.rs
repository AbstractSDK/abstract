use abstract_std::{
    ans_host::state::{ASSET_ADDRESSES, CHANNELS},
    objects::{
        account::ACCOUNT_ID,
        common_namespace::{ADMIN_NAMESPACE, OWNERSHIP_STORAGE_KEY},
        gov_type::GovernanceDetails,
        ownership::Ownership,
        AccountId, AssetEntry, ChannelEntry,
    },
    version_control::{state::ACCOUNT_ADDRESSES, AccountBase},
};
use cosmwasm_std::{testing::MockApi, Addr};
use cw_asset::AssetInfo;
use cw_storage_plus::Item;

use crate::prelude::*;

/// A mock querier setup with the proper responses for proxy/manager/accountId.
pub fn mocked_account_querier_builder() -> AbstractMockQuerierBuilder {
    AbstractMockQuerierBuilder::default().account(TEST_MANAGER, TEST_PROXY, TEST_ACCOUNT_ID)
}

pub struct AbstractMockQuerierBuilder {
    builder: MockQuerierBuilder,
    version_control: Addr,
}

impl Default for AbstractMockQuerierBuilder {
    fn default() -> Self {
        Self {
            builder: MockQuerierBuilder::default(),
            version_control: MockApi::default().addr_make(TEST_VERSION_CONTROL),
        }
    }
}
// ANCHOR: account
impl AbstractMockQuerierBuilder {
    /// Mock the existence of an Account by setting the Account id for the proxy and manager along with registering the account to version control.
    pub fn account(mut self, manager: &str, proxy: &str, account_id: AccountId) -> Self {
        let mockapi = MockApi::default();
        let manager_addr = mockapi.addr_make(manager);
        let proxy_addr = mockapi.addr_make(proxy);
        let owner = mockapi.addr_make(OWNER);

        self.builder = self
            .builder
            .with_contract_item(proxy_addr.as_str(), ACCOUNT_ID, &account_id)
            .with_contract_item(manager_addr.as_str(), ACCOUNT_ID, &account_id)
            .with_contract_item(
                proxy_addr.as_str(),
                Item::new(ADMIN_NAMESPACE),
                &Some(manager_addr.clone()),
            )
            // Setup the account owner as the test owner
            .with_contract_item(
                manager_addr.as_str(),
                Item::new(OWNERSHIP_STORAGE_KEY),
                &Some(Ownership {
                    owner: GovernanceDetails::Monarchy { monarch: owner },
                    pending_owner: None,
                    pending_expiry: None,
                }),
            )
            .with_contract_map_entry(
                self.version_control.as_str(),
                ACCOUNT_ADDRESSES,
                (
                    &account_id,
                    AccountBase {
                        manager: manager_addr,
                        proxy: proxy_addr,
                    },
                ),
            );

        self
    }
    // ANCHOR_END: account

    /// Add mock assets into ANS
    pub fn assets(mut self, assets: Vec<(&AssetEntry, AssetInfo)>) -> Self {
        self.builder =
            self.builder
                .with_contract_map_entries(TEST_ANS_HOST, ASSET_ADDRESSES, assets);

        self
    }

    pub fn channels(mut self, channels: Vec<(&ChannelEntry, String)>) -> Self {
        self.builder = self
            .builder
            .with_contract_map_entries(TEST_ANS_HOST, CHANNELS, channels);

        self
    }

    /// Change the version control address. Any account added after this will be registered to this address.
    pub fn set_version_control(mut self, version_control: Addr) -> Self {
        self.version_control = version_control;
        self
    }

    pub fn ans(self, _ans: MockAnsHost) -> Self {
        self
    }

    pub fn builder(self) -> MockQuerierBuilder {
        self.builder
    }

    pub fn build(self) -> MockQuerier {
        self.builder.build()
    }
}
