pub(crate) mod abstract_mock_querier;
pub mod map_tester;
pub mod mock_ans;
pub(crate) mod mock_querier;

use abstract_std::{manager::state::ACCOUNT_MODULES, objects::account::TEST_ACCOUNT_ID, proxy::state::ACCOUNT_ID, version_control::state::ACCOUNT_ADDRESSES};
use cosmwasm_std::{
    from_json, testing::{MockApi, MockQuerier, MockStorage}, to_json_binary, Addr, Binary, Empty, OwnedDeps
};
pub use mock_ans::MockAnsHost;
pub use mock_querier::{
    map_key, raw_map_key, wrap_querier, MockQuerierBuilder, MockQuerierOwnership,
};
use module::{TEST_MODULE_ID, TEST_MODULE_RESPONSE};
use prelude::{AbstractMockAddrs, AbstractMockQuerierBuilder};
pub type MockDeps = OwnedDeps<MockStorage, MockApi, MockQuerier>;

/// A mock querier that returns the following responses for the following **RAW** contract -> queries:
/// - TEST_PROXY
///   - "admin" -> TEST_MANAGER
/// - TEST_MANAGER
///   - "modules:TEST_MODULE_ID" -> TEST_MODULE_ADDRESS
///   - "account_id" -> TEST_ACCOUNT_ID
/// - TEST_VERSION_CONTROL
///   - "account" -> { TEST_PROXY, TEST_MANAGER }
pub fn mock_querier(mock_api: MockApi) -> MockQuerier {
    let raw_handler = move |contract: &Addr, key: &Binary| {
        // TODO: should we do something with the key?
        let _str_key = std::str::from_utf8(key.as_slice()).unwrap();
        let abstr = AbstractMockAddrs::new(mock_api);

        if contract == abstr.account.addr() {
            // Return the default value
            Ok(Binary::default())
        } else if contract == abstr.version_control {
            // Default value
            Ok(Binary::default())
        } else {
            Err("unexpected contract".to_string())
        }
    };
    let abstr = AbstractMockAddrs::new(mock_api);

    MockQuerierBuilder::default()
        .with_fallback_raw_handler(raw_handler)
        .with_contract_map_entry(
            &abstr.version_control,
            ACCOUNT_ADDRESSES,
            (&TEST_ACCOUNT_ID, abstr.account.clone()),
        )
        .with_contract_item(abstr.account.addr(), ACCOUNT_ID, &TEST_ACCOUNT_ID)
        .with_smart_handler(&abstr.module_address, |msg| {
            let Empty {} = from_json(msg).unwrap();
            Ok(to_json_binary(TEST_MODULE_RESPONSE).unwrap())
        })
        .with_contract_map_entry(
            abstr.account.addr(),
            ACCOUNT_MODULES,
            (TEST_MODULE_ID, abstr.module_address),
        )
        .build()
}


/// Abstract-specific mock dependencies. 
/// 
/// Sets the required queries for native contracts and the root Abstract Account.
pub fn mock_dependencies() -> MockDeps {
    let api = MockApi::default();
    let querier = mock_querier(api.clone());

    OwnedDeps {
        storage: MockStorage::default(),
        api,
        querier,
        custom_query_type: std::marker::PhantomData,
    }
}

/// use the package version as test version, breaks tests otherwise.
pub const TEST_VERSION: &str = env!("CARGO_PKG_VERSION");
pub mod addresses {
    use abstract_std::version_control::Account;
    use cosmwasm_std::{testing::MockApi, Addr};

    // Test addr makers
    const OWNER: &str = "owner";
    const TEST_ACCOUNT: &str = "account_address";
    const TEST_ANS_HOST: &str = "test_ans_host_address";
    const TEST_VERSION_CONTROL: &str = "version_control_address";
    const TEST_ACCOUNT_FACTORY: &str = "account_factory_address";
    const TEST_MODULE_FACTORY: &str = "module_factory_address";
    const TEST_MODULE_ADDRESS: &str = "test_module_address";
    // set in cosmwasm_std::MockApi
    const ENV_CONTRACT_ADDRESS: &str = "cosmos2address";

    pub fn test_account_base(mock_api: MockApi) -> Account {
        Account::new(mock_api.addr_make(ENV_CONTRACT_ADDRESS))
    }

    impl AbstractMockAddrs {
        pub fn new(mock_api: MockApi) -> AbstractMockAddrs {
            AbstractMockAddrs {
                owner: mock_api.addr_make(OWNER),
                ans_host: mock_api.addr_make(TEST_ANS_HOST),
                version_control: mock_api.addr_make(TEST_VERSION_CONTROL),
                account_factory: mock_api.addr_make(TEST_ACCOUNT_FACTORY),
                module_factory: mock_api.addr_make(TEST_MODULE_FACTORY),
                module_address: mock_api.addr_make(TEST_MODULE_ADDRESS),
                account: test_account_base(mock_api),
            }
        }
    }

    #[derive(Debug, Clone)]
    pub struct AbstractMockAddrs {
        pub owner: Addr,
        pub ans_host: Addr,
        pub version_control: Addr,
        #[deprecated(note = "Account factory will be removed")]
        pub account_factory: Addr,
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

pub mod prelude {
    pub use abstract_mock_querier::AbstractMockQuerierBuilder;
    pub use abstract_std::objects::account::TEST_ACCOUNT_ID;
    pub use addresses::*;
    pub use ans::*;
    pub use cosmwasm_std::{
        from_json,
        testing::{MockApi as CwMockApi, MockQuerier, MockStorage},
        to_json_binary,
    };
    pub use mock_querier::{map_key, raw_map_key, wrap_querier, MockQuerierBuilder};
    pub use module::*;
    pub use super::{mock_dependencies, mock_querier};

    use super::*;
    pub use super::{MockAnsHost, MockDeps, TEST_VERSION};
}
