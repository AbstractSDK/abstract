use crate::{addresses::*, mock_ans::MockAnsHost, MockQuerierBuilder};
use abstract_core::objects::common_namespace::OWNERSHIP_STORAGE_KEY;
use abstract_core::{
    ans_host::state::ASSET_ADDRESSES,
    objects::{common_namespace::ADMIN_NAMESPACE, core::ACCOUNT_ID, AssetEntry},
    version_control::{state::ACCOUNT_ADDRESSES, AccountBase},
};
use cosmwasm_std::{testing::MockQuerier, Addr};
use cw_asset::AssetInfo;
use cw_ownable::Ownership;
use cw_storage_plus::Item;

/// A mock querier setup with the proper responses for proxy/manager/accountId.
pub fn mocked_account_querier_builder() -> AbstractMockQuerierBuilder {
    AbstractMockQuerierBuilder::default().account(TEST_MANAGER, TEST_PROXY, TEST_ACCOUNT_ID)
}

pub struct AbstractMockQuerierBuilder {
    builder: MockQuerierBuilder,
    version_control: &'static str,
}

impl Default for AbstractMockQuerierBuilder {
    fn default() -> Self {
        Self {
            builder: MockQuerierBuilder::default(),
            version_control: TEST_VERSION_CONTROL,
        }
    }
}

impl AbstractMockQuerierBuilder {
    /// Mock the existence of an Account by setting the Account id for the proxy and manager along with registering the account to version control.
    pub fn account(mut self, manager: &str, proxy: &str, account_id: u32) -> Self {
        self.builder = self
            .builder
            .with_contract_item(proxy, ACCOUNT_ID, &account_id)
            .with_contract_item(manager, ACCOUNT_ID, &account_id)
            .with_contract_item(
                proxy,
                Item::new(ADMIN_NAMESPACE),
                &Some(Addr::unchecked(manager)),
            )
            // Setup the account owner as the test owner
            .with_contract_item(
                manager,
                Item::new(OWNERSHIP_STORAGE_KEY),
                &Some(Ownership {
                    owner: Some(Addr::unchecked(TEST_OWNER)),
                    pending_owner: None,
                    pending_expiry: None,
                }),
            )
            .with_contract_map_entry(
                self.version_control,
                ACCOUNT_ADDRESSES,
                (
                    account_id,
                    AccountBase {
                        manager: Addr::unchecked(manager),
                        proxy: Addr::unchecked(proxy),
                    },
                ),
            );

        self
    }

    /// Add mock assets into ANS
    pub fn assets(mut self, assets: Vec<(&AssetEntry, AssetInfo)>) -> Self {
        self.builder =
            self.builder
                .with_contract_map_entries(TEST_ANS_HOST, ASSET_ADDRESSES, assets);

        self
    }

    /// Change the version control address. Any account added after this will be registered to this address.
    pub fn set_version_control(mut self, version_control: &'static str) -> Self {
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
