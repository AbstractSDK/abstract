pub(crate) mod abstract_mock_querier;
pub mod map_tester;
pub mod mock_ans;
pub(crate) mod mock_querier;

use abstract_std::account::{ConfigResponse as AccountConfigResponse, QueryMsg as AccountQueryMsg};
use abstract_std::objects::ABSTRACT_ACCOUNT_ID;
use abstract_std::{
    account::state::ACCOUNT_ID,
    account::state::ACCOUNT_MODULES,
    objects::{
        module::{ModuleInfo, ModuleVersion},
        module_reference::ModuleReference,
        ownership,
    },
    version_control::state::{ACCOUNT_ADDRESSES, REGISTERED_MODULES},
    ACCOUNT,
};
use cosmwasm_std::{
    from_json,
    testing::{MockApi, MockQuerier, MockStorage},
    to_json_binary, Addr, Binary, Empty, OwnedDeps,
};
pub use mock_ans::MockAnsHost;
pub use mock_querier::{
    map_key, raw_map_key, wrap_querier, MockQuerierBuilder, MockQuerierOwnership,
};
use module::{TEST_MODULE_ID, TEST_MODULE_RESPONSE};
use prelude::*;
pub type MockDeps = OwnedDeps<MockStorage, MockApi, MockQuerier>;

pub fn abstract_mock_querier_builder(mock_api: MockApi) -> MockQuerierBuilder {
    let raw_handler = move |contract: &Addr, key: &Binary| {
        // TODO: should we do something with the key?
        let str_key = std::str::from_utf8(key.as_slice()).unwrap();
        let abstr = AbstractMockAddrs::new(mock_api);

        if contract == abstr.account.addr() {
            // Return the default value
            Ok(Binary::default())
        } else if contract == abstr.version_control {
            // Default value
            Ok(Binary::default())
        } else {
            Err(format!(
                "attempt to query {} with key {}",
                contract, str_key
            ))
        }
    };
    let abstr = AbstractMockAddrs::new(mock_api);

    MockQuerierBuilder::new(mock_api)
        .with_fallback_raw_handler(raw_handler)
        .with_contract_map_entry(
            &abstr.version_control,
            ACCOUNT_ADDRESSES,
            (&ABSTRACT_ACCOUNT_ID, abstr.account.clone()),
        )
        .with_contract_map_entry(
            &abstr.version_control,
            REGISTERED_MODULES,
            (
                &ModuleInfo::from_id(ACCOUNT, ModuleVersion::Version(TEST_VERSION.into())).unwrap(),
                ModuleReference::Account(1),
            ),
        )
        .with_contract_item(abstr.account.addr(), ACCOUNT_ID, &ABSTRACT_ACCOUNT_ID)
        .with_contract_version(abstr.account.addr(), ACCOUNT, TEST_VERSION)
        .with_smart_handler(&abstr.module_address, |msg| {
            let Empty {} = from_json(msg).unwrap();
            Ok(to_json_binary(TEST_MODULE_RESPONSE).unwrap())
        })
        .with_contract_map_entry(
            abstr.account.addr(),
            ACCOUNT_MODULES,
            (TEST_MODULE_ID, abstr.module_address),
        )
        .with_smart_handler(abstr.account.addr(), move |msg| {
            let abstr = AbstractMockAddrs::new(mock_api);
            match from_json(msg).unwrap() {
                AccountQueryMsg::Config {} => {
                    let resp = AccountConfigResponse {
                        version_control_address: abstr.version_control,
                        module_factory_address: abstr.module_factory,
                        account_id: ABSTRACT_ACCOUNT_ID, // mock value, not used
                        is_suspended: false,
                        whitelisted_addresses: vec![],
                    };
                    Ok(to_json_binary(&resp).unwrap())
                }
                AccountQueryMsg::Ownership {} => {
                    let resp = ownership::Ownership {
                        owner: ownership::GovernanceDetails::Monarchy {
                            monarch: abstr.owner,
                        },
                        pending_expiry: None,
                        pending_owner: None,
                    };
                    Ok(to_json_binary(&resp).unwrap())
                }
                _ => panic!("unexpected message"),
            }
        })
        .with_owner(abstr.account.addr(), Some(&abstr.owner))
}

/// A mock querier that returns the following responses for the following **RAW** contract -> queries:
/// - TEST_PROXY
///   - "admin" -> TEST_MANAGER
/// - TEST_MANAGER
///   - "modules:TEST_MODULE_ID" -> TEST_MODULE_ADDRESS
///   - "account_id" -> TEST_ACCOUNT_ID
/// - TEST_VERSION_CONTROL
///   - "account" -> { TEST_PROXY, TEST_MANAGER }
pub fn abstract_mock_querier(mock_api: MockApi) -> MockQuerier {
    abstract_mock_querier_builder(mock_api).build()
}

/// use the package version as test version, breaks tests otherwise.
pub const TEST_VERSION: &str = env!("CARGO_PKG_VERSION");
pub mod addresses {
    use abstract_std::{native_addrs, version_control::Account};
    use cosmwasm_std::{testing::MockApi, Addr, Api, CanonicalAddr};

    // Test addr makers
    const ADMIN_ACCOUNT: &str = "admin_account_address";
    const TEST_ACCOUNT: &str = "account_address";

    pub fn admin_account(mock_api: MockApi) -> Account {
        Account::new(mock_api.addr_make(ADMIN_ACCOUNT))
    }

    // TODO: remove it
    pub fn test_account_base(mock_api: MockApi) -> Account {
        Account::new(mock_api.addr_make(TEST_ACCOUNT))
    }

    impl AbstractMockAddrs {
        pub fn new(mock_api: MockApi) -> AbstractMockAddrs {
            AbstractMockAddrs {
                owner: mock_api
                    .addr_humanize(&CanonicalAddr::from(native_addrs::TEST_ABSTRACT_CREATOR))
                    .unwrap(),
                ans_host: mock_api
                    .addr_humanize(&CanonicalAddr::from(native_addrs::ANS_ADDR))
                    .unwrap(),
                version_control: mock_api
                    .addr_humanize(&CanonicalAddr::from(native_addrs::VERSION_CONTROL_ADDR))
                    .unwrap(),
                module_factory: mock_api
                    .addr_humanize(&CanonicalAddr::from(native_addrs::MODULE_FACTORY_ADDR))
                    .unwrap(),
                module_address: mock_api
                    .addr_humanize(&CanonicalAddr::from(native_addrs::MODULE_FACTORY_ADDR))
                    .unwrap(),
                account: test_account_base(mock_api),
            }
        }
    }

    #[derive(Debug, Clone)]
    pub struct AbstractMockAddrs {
        pub owner: Addr,
        pub ans_host: Addr,
        pub version_control: Addr,
        pub module_factory: Addr,
        pub module_address: Addr,
        pub account: Account,
    }
}

pub mod ans {
    pub const TEST_CHAIN: &str = "chain";
    pub const TEST_DEX: &str = "test_dex";
    pub const TEST_ASSET_1: &str = "chain>asset1";
    pub const TEST_ASSET_2: &str = "chain>asset2";
    pub const TEST_LP_TOKEN_NAME: &str = "test_dex/chain>asset1,chain>asset2";
    pub const TEST_UNIQUE_ID: u64 = 69u64;
    pub const TTOKEN: &str = "test_token";
    pub const EUR_USD_PAIR: &str = "dex:eur_usd_pair";
    pub const EUR_USD_LP: &str = "dex/eur,usd";
    pub const TTOKEN_EUR_PAIR: &str = "dex:wynd_eur_pair";
    pub const TTOKEN_EUR_LP: &str = "dex/wynd,eur";
    pub const EUR: &str = "eur";
    pub const USD: &str = "usd";
}

pub mod module {
    pub const TEST_MODULE_ID: &str = "tester:test-module-id";
    pub const TEST_WITH_DEP_MODULE_ID: &str = "tester-dependency:test-depending-module-id";
    pub const TEST_WITH_DEP_NAMESPACE: &str = "tester-dependency";
    pub const TEST_MODULE_NAME: &str = "test-module-id";
    pub const TEST_NAMESPACE: &str = "tester";

    pub const TEST_MODULE_RESPONSE: &str = "test_module_response";
}

pub mod mock_bech32 {
    use abstract_std::native_addrs::TEST_ABSTRACT_CREATOR;
    use cosmwasm_std::{Addr, Api, CanonicalAddr};
    use cw_orch::mock::MockBech32;

    pub fn mock_bech32_sender(bech32: &MockBech32) -> Addr {
        bech32
            .app
            .borrow()
            .api()
            .addr_humanize(&CanonicalAddr::from(TEST_ABSTRACT_CREATOR))
            .unwrap()
    }
}

pub mod prelude {
    pub use super::{abstract_mock_querier, abstract_mock_querier_builder};
    pub use abstract_mock_querier::AbstractMockQuerier;
    use abstract_std::objects::{AccountId, AccountTrace};
    pub use addresses::*;
    pub use ans::*;
    pub use cosmwasm_std::{
        from_json,
        testing::{MockApi as CwMockApi, MockQuerier, MockStorage},
        to_json_binary,
    };
    pub use mock_bech32::mock_bech32_sender;
    pub use mock_querier::{map_key, raw_map_key, wrap_querier, MockQuerierBuilder};
    pub use module::*;

    use super::*;
    pub use super::{MockAnsHost, MockDeps, TEST_VERSION};
    pub const TEST_ACCOUNT_ID: AccountId = AccountId::const_new(1, AccountTrace::Local);
}
