use std::{collections::HashMap, ops::Deref};

use abstract_std::{
    manager::state::{ACCOUNT_ID, ACCOUNT_MODULES},
    objects::{
        common_namespace::OWNERSHIP_STORAGE_KEY, gov_type::GovernanceDetails, ownership::Ownership,
    },
    version_control::state::ACCOUNT_ADDRESSES,
};
use cosmwasm_std::{
    Addr, Binary, ContractInfoResponse, ContractResult, Empty, QuerierWrapper, SystemResult,
    WasmQuery,
};
use cw2::{ContractVersion, CONTRACT};
use cw_storage_plus::{Item, Map, PrimaryKey};
use serde::{de::DeserializeOwned, Serialize};

use crate::prelude::*;

type BinaryQueryResult = Result<Binary, String>;
type ContractAddr = String;
type AdminAddr = String;
type FallbackHandler = dyn for<'a> Fn(&'a str, &'a Binary) -> BinaryQueryResult;
type SmartHandler = dyn for<'a> Fn(&'a Binary) -> BinaryQueryResult;
type RawHandler = dyn for<'a> Fn(&'a str) -> BinaryQueryResult;

/// [`MockQuerierBuilder`] is a helper to build a [`MockQuerier`].
/// Usage:
///
/// ```
/// use cosmwasm_std::{from_json, to_json_binary};
/// use abstract_testing::MockQuerierBuilder;
/// use cosmwasm_std::testing::MockQuerier;
/// use abstract_sdk::mock_module::MockModuleExecuteMsg;
///
/// let querier = MockQuerierBuilder::default().with_smart_handler("contract_address", |msg| {
///    // handle the message
///     let res = match from_json::<MockModuleExecuteMsg>(msg).unwrap() {
///         // handle the message
///        _ => panic!("unexpected message"),
///    };
///
///   Ok(to_json_binary(&msg).unwrap())
/// }).build();
/// ```
pub struct MockQuerierBuilder {
    base: MockQuerier,
    fallback_raw_handler: Box<FallbackHandler>,
    fallback_smart_handler: Box<FallbackHandler>,
    smart_handlers: HashMap<ContractAddr, Box<SmartHandler>>,
    raw_handlers: HashMap<ContractAddr, Box<RawHandler>>,
    raw_mappings: HashMap<ContractAddr, HashMap<Binary, Binary>>,
    contract_admin: HashMap<ContractAddr, AdminAddr>,
}

impl Default for MockQuerierBuilder {
    /// Create a default
    fn default() -> Self {
        let raw_fallback: fn(&str, &Binary) -> BinaryQueryResult = |addr, key| {
            let str_key = std::str::from_utf8(&key.0).unwrap();
            Err(format!(
                "No raw query handler for {addr:?} with key {str_key:?}"
            ))
        };
        let smart_fallback: fn(&str, &Binary) -> BinaryQueryResult = |addr, key| {
            let str_key = std::str::from_utf8(&key.0).unwrap();
            Err(format!(
                "unexpected smart-query on contract: {addr:?} {str_key:?}"
            ))
        };

        Self {
            base: MockQuerier::default(),
            fallback_raw_handler: Box::from(raw_fallback),
            fallback_smart_handler: Box::from(smart_fallback),
            smart_handlers: HashMap::default(),
            raw_handlers: HashMap::default(),
            raw_mappings: HashMap::default(),
            contract_admin: HashMap::default(),
        }
    }
}

pub fn map_key<'a, K, V>(map: &Map<'a, K, V>, key: K) -> String
where
    V: Serialize + DeserializeOwned,
    K: PrimaryKey<'a>,
{
    String::from_utf8(raw_map_key(map, key)).unwrap()
}

pub fn raw_map_key<'a, K, V>(map: &Map<'a, K, V>, key: K) -> Vec<u8>
where
    V: Serialize + DeserializeOwned,
    K: PrimaryKey<'a>,
{
    map.key(key).deref().to_vec()
}

impl MockQuerierBuilder {
    pub fn with_fallback_smart_handler<SH>(mut self, handler: SH) -> Self
    where
        SH: 'static + Fn(&str, &Binary) -> BinaryQueryResult,
    {
        self.fallback_smart_handler = Box::new(handler);
        self
    }

    pub fn with_fallback_raw_handler<RH>(mut self, handler: RH) -> Self
    where
        RH: 'static + Fn(&str, &Binary) -> BinaryQueryResult,
    {
        self.fallback_raw_handler = Box::new(handler);
        self
    }

    /// Add a smart query contract handler to the mock querier. The handler will be called when the
    /// contract address is queried with the given message.
    /// Usage:
    /// ```rust
    /// use cosmwasm_std::{from_json, to_json_binary};
    /// use abstract_testing::MockQuerierBuilder;
    /// use cosmwasm_std::testing::MockQuerier;
    /// use abstract_sdk::mock_module::{MockModuleQueryMsg, MockModuleQueryResponse};
    ///
    /// let querier = MockQuerierBuilder::default().with_smart_handler("contract_address", |msg| {
    ///    // handle the message
    ///     let res = match from_json::<MockModuleQueryMsg>(msg).unwrap() {
    ///         // handle the message
    ///         MockModuleQueryMsg =>
    ///                         return to_json_binary(&MockModuleQueryResponse {}).map_err(|e| e.to_string())
    ///    };
    /// }).build();
    ///
    /// ```
    pub fn with_smart_handler<SH>(mut self, contract: &str, handler: SH) -> Self
    where
        SH: 'static + Fn(&Binary) -> BinaryQueryResult,
    {
        self.smart_handlers
            .insert(contract.to_string(), Box::new(handler));
        self
    }

    /// Add a raw query contract handler to the mock querier. The handler will be called when the
    /// contract address is queried with the given message.
    /// Usage:
    ///
    /// ```rust
    /// use cosmwasm_std::{from_json, to_json_binary};
    /// use abstract_testing::MockQuerierBuilder;
    /// use cosmwasm_std::testing::MockQuerier;
    /// use abstract_sdk::mock_module::{MockModuleQueryMsg, MockModuleQueryResponse};
    ///
    /// let querier = MockQuerierBuilder::default().with_raw_handler("contract1", |key: &str| {
    ///     // Example: Let's say, in the raw storage, the key "the key" maps to the value "the value"
    ///     match key {
    ///         "the key" => to_json_binary("the value").map_err(|e| e.to_string()),
    ///         _ => to_json_binary("").map_err(|e| e.to_string())
    ///     }
    /// }).build();
    /// ```
    pub fn with_raw_handler<RH>(mut self, contract: &str, handler: RH) -> Self
    where
        RH: 'static + Fn(&str) -> BinaryQueryResult,
    {
        self.raw_handlers
            .insert(contract.to_string(), Box::new(handler));
        self
    }

    fn insert_contract_key_value(&mut self, contract: &str, key: Vec<u8>, value: Binary) {
        let raw_map = self.raw_mappings.entry(contract.to_string()).or_default();
        raw_map.insert(Binary(key), value);
    }

    /// Add a map entry to the querier for the given contract.
    /// ```rust
    /// use cw_storage_plus::Map;
    /// use abstract_testing::MockQuerierBuilder;
    ///
    /// const MAP: Map<String, String> = Map::new("map");
    ///
    /// MockQuerierBuilder::default()
    ///     .with_contract_map_entry(
    ///     "contract_address",
    ///     MAP,
    ///     ("key".to_string(), "value".to_string())
    /// );
    pub fn with_contract_map_entry<'a, K, V>(
        self,
        contract: &str,
        cw_map: Map<'a, K, V>,
        entry: (K, V),
    ) -> Self
    where
        K: PrimaryKey<'a>,
        V: Serialize + DeserializeOwned,
    {
        self.with_contract_map_entries(contract, cw_map, vec![entry])
    }

    pub fn with_contract_map_entries<'a, K, V>(
        mut self,
        contract: &str,
        cw_map: Map<'a, K, V>,
        entries: Vec<(K, V)>,
    ) -> Self
    where
        K: PrimaryKey<'a>,
        V: Serialize + DeserializeOwned,
    {
        for (key, value) in entries {
            self.insert_contract_key_value(
                contract,
                raw_map_key(&cw_map, key),
                to_json_binary(&value).unwrap(),
            );
        }

        self
    }

    /// Add an empty map key to the querier for the given contract.
    /// This is useful when you want the item to exist, but not have a value.
    pub fn with_contract_map_key<'a, K, V>(
        mut self,
        contract: &str,
        cw_map: Map<'a, K, V>,
        key: K,
    ) -> Self
    where
        K: PrimaryKey<'a>,
        V: Serialize + DeserializeOwned,
    {
        self.insert_contract_key_value(contract, raw_map_key(&cw_map, key), Binary(vec![]));

        self
    }

    /// Add an empty item key to the querier for the given contract.
    /// This is useful when you want the item to exist, but not have a value.
    pub fn with_empty_contract_item<T>(mut self, contract: &str, cw_item: Item<T>) -> Self
    where
        T: Serialize + DeserializeOwned,
    {
        self.insert_contract_key_value(contract, cw_item.as_slice().to_vec(), Binary(vec![]));

        self
    }

    /// Include a contract item in the mock querier.
    /// ```rust
    /// use cw_storage_plus::Item;
    /// use abstract_testing::MockQuerierBuilder;
    ///
    /// const ITEM: Item<String> = Item::new("item");
    ///
    /// MockQuerierBuilder::default()
    ///     .with_contract_item(
    ///     "contract_address",
    ///     ITEM,
    ///     &"value".to_string(),
    /// );
    /// ```
    pub fn with_contract_item<T>(mut self, contract: &str, cw_item: Item<T>, value: &T) -> Self
    where
        T: Serialize + DeserializeOwned,
    {
        self.insert_contract_key_value(
            contract,
            cw_item.as_slice().to_vec(),
            to_json_binary(value).unwrap(),
        );

        self
    }

    /// Add a specific version of the contract to the mock querier.
    /// ```rust
    /// use abstract_testing::MockQuerierBuilder;
    ///
    /// MockQuerierBuilder::default()
    ///    .with_contract_version("contract_address", "v1.0.0");
    /// ```
    pub fn with_contract_version(self, contract: &str, version: impl ToString) -> Self {
        self.with_contract_item(
            contract,
            CONTRACT,
            &ContractVersion {
                contract: contract.to_string(),
                version: version.to_string(),
            },
        )
    }
    /// set the SDK-level contract admin for a contract.
    pub fn with_contract_admin(mut self, contract: impl ToString, admin: impl ToString) -> Self {
        self.contract_admin
            .insert(contract.to_string(), admin.to_string());
        self
    }

    /// Build the [`MockQuerier`].
    pub fn build(mut self) -> MockQuerier {
        self.base.update_wasm(move |wasm| {
            let res = match wasm {
                WasmQuery::Raw { contract_addr, key } => {
                    let str_key = std::str::from_utf8(&key.0).unwrap();

                    // First check for raw mappings
                    if let Some(raw_map) = self.raw_mappings.get(contract_addr.as_str()) {
                        if let Some(value) = raw_map.get(key) {
                            return SystemResult::Ok(ContractResult::Ok(value.clone()));
                        }
                    }

                    // Then check the handlers
                    let raw_handler = self.raw_handlers.get(contract_addr.as_str());

                    match raw_handler {
                        Some(handler) => (*handler)(str_key),
                        None => (*self.fallback_raw_handler)(contract_addr.as_str(), key),
                    }
                }
                WasmQuery::Smart { contract_addr, msg } => {
                    let contract_handler = self.smart_handlers.get(contract_addr.as_str());

                    let res = match contract_handler {
                        Some(handler) => (*handler)(msg),
                        None => (*self.fallback_smart_handler)(contract_addr.as_str(), msg),
                    };
                    res
                }
                WasmQuery::ContractInfo { contract_addr } => {
                    let mut info = ContractInfoResponse::default();
                    info.admin = self.contract_admin.get(contract_addr).cloned();
                    Ok(to_json_binary(&info).unwrap())
                }
                unexpected => panic!("Unexpected query: {unexpected:?}"),
            };

            match res {
                Ok(res) => SystemResult::Ok(ContractResult::Ok(res)),
                Err(e) => SystemResult::Ok(ContractResult::Err(e)),
            }
        });
        self.base
    }
}

pub trait MockQuerierOwnership {
    /// Add the [`cw_gov_ownable::Ownership`] to the querier.
    fn with_owner(self, contract: &str, owner: Option<impl ToString>) -> Self;
}

impl MockQuerierOwnership for MockQuerierBuilder {
    fn with_owner(mut self, contract: &str, owner: Option<impl ToString>) -> Self {
        let owner = if let Some(owner) = owner {
            GovernanceDetails::Monarchy {
                monarch: Addr::unchecked(owner.to_string()),
            }
        } else {
            GovernanceDetails::Renounced {}
        };
        self = self.with_contract_item(
            contract,
            Item::new(OWNERSHIP_STORAGE_KEY),
            &Ownership {
                owner,
                pending_owner: None,
                pending_expiry: None,
            },
        );
        self
    }
}

/// A mock querier that returns the following responses for the following **RAW** contract -> queries:
/// - TEST_PROXY
///   - "admin" -> TEST_MANAGER
/// - TEST_MANAGER
///   - "modules:TEST_MODULE_ID" -> TEST_MODULE_ADDRESS
///   - "account_id" -> TEST_ACCOUNT_ID
/// - TEST_VERSION_CONTROL
///   - "account" -> { TEST_PROXY, TEST_MANAGER }
pub fn mock_querier() -> MockQuerier {
    let raw_handler = |contract: &str, key: &Binary| {
        let _str_key = std::str::from_utf8(&key.0).unwrap();
        match contract {
            TEST_PROXY => Err("unexpected key".to_string()),
            TEST_MANAGER => {
                // Return the default value
                Ok(Binary(vec![]))
            }
            TEST_VERSION_CONTROL => {
                // Default value
                Ok(Binary(vec![]))
            }
            _ => Err("unexpected contract".to_string()),
        }
    };

    MockQuerierBuilder::default()
        .with_fallback_raw_handler(raw_handler)
        .with_contract_map_entry(
            TEST_VERSION_CONTROL,
            ACCOUNT_ADDRESSES,
            (&TEST_ACCOUNT_ID, test_account_base()),
        )
        .with_contract_item(
            TEST_PROXY,
            Item::new("admin"),
            &Some(Addr::unchecked(TEST_MANAGER)),
        )
        .with_contract_item(TEST_MANAGER, ACCOUNT_ID, &TEST_ACCOUNT_ID)
        .with_smart_handler(TEST_MODULE_ADDRESS, |msg| {
            let Empty {} = from_json(msg).unwrap();
            Ok(to_json_binary(TEST_MODULE_RESPONSE).unwrap())
        })
        .with_contract_map_entry(
            TEST_MANAGER,
            ACCOUNT_MODULES,
            (TEST_MODULE_ID, Addr::unchecked(TEST_MODULE_ADDRESS)),
        )
        .build()
}

pub fn wrap_querier(querier: &MockQuerier) -> QuerierWrapper<'_, Empty> {
    QuerierWrapper::<Empty>::new(querier)
}

#[cfg(test)]
mod tests {
    use abstract_std::{
        manager::state::ACCOUNT_MODULES, proxy::state::ACCOUNT_ID,
        version_control::state::ACCOUNT_ADDRESSES,
    };
    use cosmwasm_std::testing::mock_dependencies;
    use speculoos::prelude::*;

    use super::*;

    mod account {
        use abstract_std::version_control::AccountBase;

        use super::*;

        #[test]
        fn should_return_account_address() {
            let mut deps = mock_dependencies();
            deps.querier = mock_querier();

            let actual = ACCOUNT_ADDRESSES.query(
                &wrap_querier(&deps.querier),
                Addr::unchecked(TEST_VERSION_CONTROL),
                &TEST_ACCOUNT_ID,
            );

            let expected = AccountBase {
                proxy: Addr::unchecked(TEST_PROXY),
                manager: Addr::unchecked(TEST_MANAGER),
            };

            assert_that!(actual).is_ok().is_some().is_equal_to(expected)
        }
    }

    mod queries {
        use super::*;

        use abstract_sdk::mock_module::{MockModuleQueryMsg, MockModuleQueryResponse};
        use cosmwasm_std::QueryRequest;

        #[test]
        fn smart_query() {
            // ## ANCHOR: smart_query
            let querier = MockQuerierBuilder::default()
                .with_smart_handler("contract_address", |msg| {
                    // handle the message
                    let MockModuleQueryMsg {} = from_json::<MockModuleQueryMsg>(msg).unwrap();
                    to_json_binary(&MockModuleQueryResponse {}).map_err(|e| e.to_string())
                })
                .build();
            // ## ANCHOR_END: smart_query

            let resp_bin = querier
                .handle_query(&QueryRequest::Wasm(WasmQuery::Smart {
                    contract_addr: "contract_address".to_string(),
                    msg: to_json_binary(&MockModuleQueryMsg {}).unwrap(),
                }))
                .unwrap()
                .unwrap();
            let resp: MockModuleQueryResponse = from_json(resp_bin).unwrap();

            assert_that!(resp).is_equal_to(MockModuleQueryResponse {})
        }

        #[test]
        fn raw_query() {
            // ## ANCHOR: raw_query
            let querier = MockQuerierBuilder::default()
                .with_raw_handler("contract_address", |key: &str| {
                    // Example: Let's say, in the raw storage, the key "the_key" maps to the value "the_value"
                    match key {
                        "the_key" => to_json_binary("the_value").map_err(|e| e.to_string()),
                        _ => to_json_binary("").map_err(|e| e.to_string()),
                    }
                })
                .build();
            // ## ANCHOR_END: raw_query

            let resp_bin = querier
                .handle_query(&QueryRequest::Wasm(WasmQuery::Raw {
                    contract_addr: "contract_address".to_string(),
                    key: Binary::from("the_key".joined_key()),
                }))
                .unwrap()
                .unwrap();
            let resp: String = from_json(resp_bin).unwrap();

            assert_that!(resp).is_equal_to("the_value".to_string())
        }
    }

    mod account_id {
        use super::*;

        #[test]
        fn should_return_test_acct_id_with_test_manager() {
            let mut deps = mock_dependencies();
            deps.querier = mock_querier();
            let actual =
                ACCOUNT_ID.query(&wrap_querier(&deps.querier), Addr::unchecked(TEST_MANAGER));

            assert_that!(actual).is_ok().is_equal_to(TEST_ACCOUNT_ID);
        }
    }

    mod account_modules {
        use super::*;

        #[test]
        fn should_return_test_module_address_for_test_module() {
            let mut deps = mock_dependencies();
            deps.querier = mock_querier();

            let actual = ACCOUNT_MODULES.query(
                &wrap_querier(&deps.querier),
                Addr::unchecked(TEST_MANAGER),
                TEST_MODULE_ID,
            );

            assert_that!(actual)
                .is_ok()
                .is_some()
                .is_equal_to(Addr::unchecked(TEST_MODULE_ADDRESS));
        }

        // #[test]
        // fn should_return_none_for_unknown_module() {
        //     let mut deps = mock_dependencies();
        //     deps.querier = querier();
        //
        //     let actual = ACCOUNT_MODULES.query(
        //         &wrap_querier(&deps.querier),
        //         Addr::unchecked(TEST_MANAGER),
        //         "unknown_module",
        //     );
        //
        //     assert_that!(actual).is_ok().is_none();
        // }
    }
}
