pub(crate) mod abstract_mock_querier;
pub mod map_tester;
pub mod mock_ans;
pub(crate) mod mock_querier;

use cosmwasm_std::{
    testing::{MockApi, MockQuerier, MockStorage},
    OwnedDeps,
};
pub use mock_ans::MockAnsHost;
pub use mock_querier::{
    map_key, mock_querier, raw_map_key, wrap_querier, MockQuerierBuilder, MockQuerierOwnership,
};
pub type MockDeps = OwnedDeps<MockStorage, MockApi, MockQuerier>;
pub const OWNER: &str = "owner";
pub mod addresses {
    use abstract_std::version_control::AccountBase;
    use cosmwasm_std::{testing::MockApi, Addr};

    /// use the package version as test version, breaks tests otherwise.
    pub const TEST_VERSION: &str = env!("CARGO_PKG_VERSION");
    pub const TEST_PROXY: &str = "proxy_address";
    pub const TEST_MANAGER: &str = "manager_address";
    pub const TEST_ANS_HOST: &str = "test_ans_host_address";
    pub const TEST_VERSION_CONTROL: &str = "version_control_address";
    pub const TEST_ACCOUNT_FACTORY: &str = "account_factory_address";
    pub const TEST_MODULE_FACTORY: &str = "module_factory_address";
    pub const TEST_MODULE_ADDRESS: &str = "test_module_address";
    pub const TEST_MODULE_ID: &str = "tester:test-module-id";
    pub const TEST_WITH_DEP_MODULE_ID: &str = "tester-dependency:test-depending-module-id";
    pub const TEST_WITH_DEP_NAMESPACE: &str = "tester-dependency";
    pub const TEST_MODULE_NAME: &str = "test-module-id";
    pub const TEST_NAMESPACE: &str = "tester";

    pub const TEST_MODULE_RESPONSE: &str = "test_module_response";

    pub const TEST_CHAIN: &str = "chain";
    pub const TEST_DEX: &str = "test_dex";
    pub const TEST_ASSET_1: &str = "chain>asset1";
    pub const TEST_ASSET_2: &str = "chain>asset2";
    pub const TEST_LP_TOKEN_NAME: &str = "test_dex/chain>asset1,chain>asset2";
    pub const TEST_LP_TOKEN_ADDR: &str = "test_dex_asset1_asset2_lp_token";
    pub const TEST_POOL_ADDR: &str = "test_pool_address";
    pub const TEST_UNIQUE_ID: u64 = 69u64;
    pub const TTOKEN: &str = "test_token";
    pub const EUR_USD_PAIR: &str = "dex:eur_usd_pair";
    pub const EUR_USD_LP: &str = "dex/eur,usd";
    pub const TTOKEN_EUR_PAIR: &str = "dex:wynd_eur_pair";
    pub const TTOKEN_EUR_LP: &str = "dex/wynd,eur";
    pub const EUR: &str = "eur";
    pub const USD: &str = "usd";

    pub fn test_account_base() -> AccountBase {
        let mock_api = MockApi::default();
        AccountBase {
            manager: mock_api.addr_make(TEST_MANAGER),
            proxy: mock_api.addr_make(TEST_PROXY),
        }
    }
}

pub mod prelude {
    pub use abstract_mock_querier::{mocked_account_querier_builder, AbstractMockQuerierBuilder};
    pub use abstract_std::objects::account::TEST_ACCOUNT_ID;
    pub use addresses::*;
    pub use cosmwasm_std::{
        from_json,
        testing::{MockApi as CwMockApi, MockQuerier, MockStorage},
        to_json_binary,
    };
    pub use mock_querier::{map_key, mock_querier, raw_map_key, wrap_querier, MockQuerierBuilder};

    use super::*;
    pub use super::{MockAnsHost, MockDeps, OWNER};
}
