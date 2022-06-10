use std::collections::HashMap;

use cosmwasm_std::testing::{mock_env, MockApi, MockStorage};
use cosmwasm_std::{Addr, Coin};

use abstract_os::native::version_control::state::Core;
use cw_multi_test::{App, AppBuilder, BankKeeper};

use crate::tests::common::{RANDOM_USER, TEST_CREATOR};

use super::os_creation::{init_os, init_primary_os};
use super::upload::upload_base_contracts;

pub struct NativeContracts {
    pub token: Addr,
    pub memory: Addr,
    pub version_control: Addr,
    pub os_factory: Addr,
    pub module_factory: Addr,
}

pub fn mock_app() -> App {
    let env = mock_env();
    let api = MockApi::default();
    let bank = BankKeeper::new();
    let storage = MockStorage::new();

    let env = mock_env();
    let api = MockApi::default();
    let bank = BankKeeper::new();

    let sender = Addr::unchecked(TEST_CREATOR);
    let random_user = Addr::unchecked(RANDOM_USER);

    let funds = vec![Coin::new(1_000_000_000, "uusd")];

    AppBuilder::new()
        .with_api(api)
        .with_block(env.block)
        .with_bank(bank)
        .with_storage(storage)
        .build(|router, _, storage| {
            router
                .bank
                .init_balance(storage, &sender, funds.clone())
                .unwrap();

            router
                .bank
                .init_balance(storage, &random_user, funds)
                .unwrap();
        })
}
