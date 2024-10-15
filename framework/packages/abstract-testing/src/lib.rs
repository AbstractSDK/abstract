pub(crate) mod abstract_mock_querier;
pub mod map_tester;
pub mod mock_ans;
pub(crate) mod mock_querier;

use abstract_std::account::{ConfigResponse as AccountConfigResponse, QueryMsg as AccountQueryMsg};
use abstract_std::objects::ABSTRACT_ACCOUNT_ID;
use abstract_std::registry;
use abstract_std::{
    account::state::ACCOUNT_ID,
    account::state::ACCOUNT_MODULES,
    objects::{
        module::{ModuleInfo, ModuleVersion},
        module_reference::ModuleReference,
        ownership,
    },
    registry::state::{ACCOUNT_ADDRESSES, REGISTERED_MODULES},
    ACCOUNT,
};
use cosmwasm_std::{
    from_json,
    testing::{MockApi, MockQuerier, MockStorage},
    to_json_binary, Addr, Binary, Empty, OwnedDeps,
};
use cosmwasm_std::{ContractInfo, Env};
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
        } else if contract == abstr.registry {
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
            &abstr.registry,
            ACCOUNT_ADDRESSES,
            (&ABSTRACT_ACCOUNT_ID, abstr.account.clone()),
        )
        .with_contract_item(
            &abstr.registry,
            registry::state::CONFIG,
            &registry::Config {
                security_disabled: true,
                namespace_registration_fee: None,
            },
        )
        .with_contract_map_entry(
            &abstr.registry,
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
                        registry_address: abstr.registry,
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
/// - ABSTRACT_ACCOUNT
///   - "admin" -> TEST_OWNER
///   - "modules:TEST_MODULE_ID" -> TEST_MODULE_ADDRESS
///   - "account_id" -> ABSTRACT_ACCOUNT_ID
/// - REGISTRY
///   - "account" -> { ABSTRACT_ACCOUNT }
pub fn abstract_mock_querier(mock_api: MockApi) -> MockQuerier {
    abstract_mock_querier_builder(mock_api).build()
}

/// cosmwasm_std::mock_env_validated(deps.api), but address generated with MockApi
pub fn mock_env_validated(mock_api: MockApi) -> Env {
    Env {
        contract: ContractInfo {
            address: mock_api.addr_make(cosmwasm_std::testing::MOCK_CONTRACT_ADDR),
        },
        ..cosmwasm_std::testing::mock_env()
    }
}

/// use the package version as test version, breaks tests otherwise.
pub const TEST_VERSION: &str = env!("CARGO_PKG_VERSION");
pub mod addresses {
    use abstract_std::{native_addrs, registry::Account};
    use cosmwasm_std::{testing::MockApi, Addr, Api};

    use crate::mock_env_validated;

    // Test addr makers
    const ADMIN_ACCOUNT: &str = "admin_account_address";
    const TEST_ACCOUNT: &str = "account_address";

    pub fn admin_account(mock_api: MockApi) -> Account {
        Account::new(mock_api.addr_make(ADMIN_ACCOUNT))
    }

    // TODO: remove it
    pub fn test_account(mock_api: MockApi) -> Account {
        Account::new(mock_api.addr_make(TEST_ACCOUNT))
    }

    impl AbstractMockAddrs {
        pub fn new(mock_api: MockApi) -> AbstractMockAddrs {
            let mock_env = mock_env_validated(mock_api);
            let hrp = native_addrs::hrp_from_env(&mock_env);

            AbstractMockAddrs {
                owner: mock_api
                    .addr_validate(&native_addrs::creator_address(hrp).unwrap())
                    .unwrap(),
                ans_host: mock_api
                    .addr_humanize(&native_addrs::ans_address(hrp, &mock_api).unwrap())
                    .unwrap(),
                registry: mock_api
                    .addr_humanize(&native_addrs::registry_address(hrp, &mock_api).unwrap())
                    .unwrap(),
                module_factory: mock_api
                    .addr_humanize(&native_addrs::module_factory_address(hrp, &mock_api).unwrap())
                    .unwrap(),
                module_address: mock_api.addr_make("module"),
                account: admin_account(mock_api),
            }
        }
    }

    #[derive(Debug, Clone)]
    pub struct AbstractMockAddrs {
        pub owner: Addr,
        pub ans_host: Addr,
        pub registry: Addr,
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
    pub use super::{abstract_mock_querier, abstract_mock_querier_builder, mock_env_validated};
    pub use abstract_mock_querier::AbstractMockQuerier;
    use abstract_std::objects::{AccountId, AccountTrace};
    pub use addresses::*;
    pub use ans::*;
    pub use cosmwasm_std::{
        from_json,
        testing::{MockApi as CwMockApi, MockQuerier, MockStorage},
        to_json_binary,
    };
    pub use mock_querier::{map_key, raw_map_key, wrap_querier, MockQuerierBuilder};
    pub use module::*;

    use super::*;
    pub use super::{MockAnsHost, MockDeps, TEST_VERSION};
    pub const TEST_ACCOUNT_ID: AccountId = AccountId::const_new(1, AccountTrace::Local);
}
