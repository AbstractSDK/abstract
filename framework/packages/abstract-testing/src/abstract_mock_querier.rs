use abstract_std::{
    account::state::{ACCOUNT_ID, CALLING_TO_AS_ADMIN},
    ans_host::state::{ASSET_ADDRESSES, CHANNELS, CONTRACT_ADDRESSES},
    objects::{
        gov_type::GovernanceDetails, ownership::Ownership,
        storage_namespaces::OWNERSHIP_STORAGE_KEY, AccountId, AssetEntry, ChannelEntry,
        ContractEntry,
    },
    registry::{state::ACCOUNT_ADDRESSES, Account},
};
use cosmwasm_std::{testing::mock_env, Addr};
use cw_asset::AssetInfo;
use cw_storage_plus::Item;

use crate::prelude::*;

pub trait AbstractMockQuerier {
    /// Mock the existence of an Account by setting the Account id for the account along with registering the account to version control.
    fn account(self, account: &Account, account_id: AccountId) -> Self;

    /// Add mock assets into ANS
    fn assets(self, assets: Vec<(&AssetEntry, AssetInfo)>) -> Self;

    fn set_account_admin_call_to(self, account: &Account) -> Self;

    fn contracts(self, contracts: Vec<(&ContractEntry, Addr)>) -> Self;

    fn channels(self, channels: Vec<(&ChannelEntry, String)>) -> Self;

    fn addrs(&self) -> AbstractMockAddrs;
}

impl AbstractMockQuerier for MockQuerierBuilder {
    /// Mock the existence of an Account by setting the Account id for the account along with registering the account to version control.
    fn account(self, account: &Account, account_id: AccountId) -> Self {
        let abstract_addrs = self.addrs();
        self.with_contract_item(account.addr(), ACCOUNT_ID, &account_id)
            // Setup the account owner as the test owner
            .with_contract_item(
                account.addr(),
                Item::new(OWNERSHIP_STORAGE_KEY),
                &Some(Ownership {
                    owner: GovernanceDetails::Monarchy {
                        monarch: abstract_addrs.owner.clone(),
                    },
                    pending_owner: None,
                    pending_expiry: None,
                }),
            )
            .with_contract_map_entry(
                &abstract_addrs.registry,
                ACCOUNT_ADDRESSES,
                (&account_id, account.clone()),
            )
            .with_contract_map_entry(
                account.addr(),
                abstract_std::account::state::ACCOUNT_MODULES,
                (TEST_MODULE_ID, abstract_addrs.module_address),
            )
    }

    fn assets(self, assets: Vec<(&AssetEntry, AssetInfo)>) -> Self {
        let abstract_addrs = self.addrs();
        self.with_contract_map_entries(&abstract_addrs.ans_host, ASSET_ADDRESSES, assets)
    }

    fn contracts(self, contracts: Vec<(&ContractEntry, Addr)>) -> Self {
        let abstract_addrs = self.addrs();

        self.with_contract_map_entries(&abstract_addrs.ans_host, CONTRACT_ADDRESSES, contracts)
    }

    fn channels(self, channels: Vec<(&ChannelEntry, String)>) -> Self {
        let abstract_addrs = self.addrs();

        self.with_contract_map_entries(&abstract_addrs.ans_host, CHANNELS, channels)
    }

    fn addrs(&self) -> AbstractMockAddrs {
        AbstractMockAddrs::new(self.api)
    }

    fn set_account_admin_call_to(self, account: &Account) -> Self {
        self.with_contract_item(
            account.addr(),
            CALLING_TO_AS_ADMIN,
            &mock_env().contract.address,
        )
    }
}
