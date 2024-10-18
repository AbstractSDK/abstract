use std::{collections::HashMap, ops::Deref};

use abstract_std::objects::{
    gov_type::GovernanceDetails, ownership::Ownership, storage_namespaces::OWNERSHIP_STORAGE_KEY,
};
use cosmwasm_std::{
    testing::MockApi, Addr, Binary, ContractInfoResponse, ContractResult, Empty, QuerierWrapper,
    SystemResult, WasmQuery,
};
use cw2::{ContractVersion, CONTRACT};
use cw_storage_plus::{Item, Map, PrimaryKey};
use serde::{de::DeserializeOwned, Serialize};

use crate::prelude::*;

type BinaryQueryResult = Result<Binary, String>;
type FallbackHandler = dyn for<'a> Fn(&'a Addr, &'a Binary) -> BinaryQueryResult;
type SmartHandler = dyn for<'a> Fn(&'a Binary) -> BinaryQueryResult;
type RawHandler = dyn for<'a> Fn(&'a str) -> BinaryQueryResult;

/// [`MockQuerierBuilder`] is a helper to build a [`MockQuerier`].
/// Usage:
///
/// ```
/// use cosmwasm_std::{from_json, to_json_binary};
/// use abstract_testing::MockQuerierBuilder;
/// use cosmwasm_std::testing::{MockQuerier, MockApi};
/// use abstract_sdk::mock_module::MockModuleExecuteMsg;
///
/// let api = MockApi::default();
/// let contract_address = api.addr_make("contract_address");
/// let querier = MockQuerierBuilder::default().with_smart_handler(&contract_address, |msg| {
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
    smart_handlers: HashMap<Addr, Box<SmartHandler>>,
    raw_handlers: HashMap<Addr, Box<RawHandler>>,
    raw_mappings: HashMap<Addr, HashMap<Binary, Binary>>,
    contract_admin: HashMap<Addr, Addr>,
    // Used for Address generation
    pub api: MockApi,
}

impl Default for MockQuerierBuilder {
    /// Create a default
    fn default() -> Self {
        Self::new(MockApi::default())
    }
}

impl MockQuerierBuilder {
    pub fn new(api: MockApi) -> Self {
        let raw_fallback: fn(&Addr, &Binary) -> BinaryQueryResult = |addr, key| {
            let str_key = std::str::from_utf8(key.as_slice()).unwrap();
            Err(format!(
                "No raw query handler for {addr:?} with key {str_key:?}"
            ))
        };
        let smart_fallback: fn(&Addr, &Binary) -> BinaryQueryResult = |addr, key| {
            let str_key = std::str::from_utf8(key.as_slice()).unwrap();
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
            api,
        }
    }
}

pub fn map_key<'a, K, V>(map: &Map<K, V>, key: K) -> String
where
    V: Serialize + DeserializeOwned,
    K: PrimaryKey<'a>,
{
    String::from_utf8(raw_map_key(map, key)).unwrap()
}

pub fn raw_map_key<'a, K, V>(map: &Map<K, V>, key: K) -> Vec<u8>
where
    V: Serialize + DeserializeOwned,
    K: PrimaryKey<'a>,
{
    map.key(key).deref().to_vec()
}

impl MockQuerierBuilder {
    pub fn with_fallback_smart_handler<SH>(mut self, handler: SH) -> Self
    where
        SH: 'static + Fn(&Addr, &Binary) -> BinaryQueryResult,
    {
        self.fallback_smart_handler = Box::new(handler);
        self
    }

    pub fn with_fallback_raw_handler<RH>(mut self, handler: RH) -> Self
    where
        RH: 'static + Fn(&Addr, &Binary) -> BinaryQueryResult,
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
    /// use cosmwasm_std::testing::{MockQuerier, MockApi};
    /// use abstract_sdk::mock_module::{MockModuleQueryMsg, MockModuleQueryResponse};
    ///
    /// let api = MockApi::default();
    /// let contract_address = api.addr_make("contract_address");
    /// let querier = MockQuerierBuilder::default().with_smart_handler(&contract_address, |msg| {
    ///    // handle the message
    ///     let res = match from_json::<MockModuleQueryMsg>(msg).unwrap() {
    ///         // handle the message
    ///         MockModuleQueryMsg =>
    ///                         return to_json_binary(&MockModuleQueryResponse {}).map_err(|e| e.to_string())
    ///    };
    /// }).build();
    ///
    /// ```
    pub fn with_smart_handler<SH>(mut self, contract: &Addr, handler: SH) -> Self
    where
        SH: 'static + Fn(&Binary) -> BinaryQueryResult,
    {
        self.smart_handlers
            .insert(contract.clone(), Box::new(handler));
        self
    }

    /// Add a raw query contract handler to the mock querier. The handler will be called when the
    /// contract address is queried with the given message.
    /// Usage:
    ///
    /// ```rust
    /// use cosmwasm_std::{from_json, to_json_binary};
    /// use abstract_testing::MockQuerierBuilder;
    /// use cosmwasm_std::testing::{MockQuerier, MockApi};
    /// use abstract_sdk::mock_module::{MockModuleQueryMsg, MockModuleQueryResponse};
    ///
    /// let api = MockApi::default();
    /// let contract_address = api.addr_make("contract1");
    /// let querier = MockQuerierBuilder::default().with_raw_handler(&contract_address, |key: &str| {
    ///     // Example: Let's say, in the raw storage, the key "the key" maps to the value "the value"
    ///     match key {
    ///         "the key" => to_json_binary("the value").map_err(|e| e.to_string()),
    ///         _ => to_json_binary("").map_err(|e| e.to_string())
    ///     }
    /// }).build();
    /// ```
    pub fn with_raw_handler<RH>(mut self, contract: &Addr, handler: RH) -> Self
    where
        RH: 'static + Fn(&str) -> BinaryQueryResult,
    {
        self.raw_handlers
            .insert(contract.clone(), Box::new(handler));
        self
    }

    fn insert_contract_key_value(&mut self, contract: &Addr, key: Vec<u8>, value: Binary) {
        let raw_map = self.raw_mappings.entry(contract.clone()).or_default();
        raw_map.insert(Binary::new(key), value);
    }

    /// Add a map entry to the querier for the given contract.
    /// ```rust
    /// use cw_storage_plus::Map;
    /// use cosmwasm_std::testing::MockApi;
    /// use abstract_testing::MockQuerierBuilder;
    ///
    /// let api = MockApi::default();
    /// let contract_address = api.addr_make("contract1");
    ///
    /// const MAP: Map<String, String> = Map::new("map");
    ///
    /// MockQuerierBuilder::default()
    ///     .with_contract_map_entry(
    ///     &contract_address,
    ///     MAP,
    ///     ("key".to_string(), "value".to_string())
    /// );
    pub fn with_contract_map_entry<'a, K, V>(
        self,
        contract: &Addr,
        cw_map: Map<K, V>,
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
        contract: &Addr,
        cw_map: Map<K, V>,
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
        contract: &Addr,
        cw_map: Map<K, V>,
        key: K,
    ) -> Self
    where
        K: PrimaryKey<'a>,
        V: Serialize + DeserializeOwned,
    {
        self.insert_contract_key_value(contract, raw_map_key(&cw_map, key), Binary::default());

        self
    }

    /// Add an empty item key to the querier for the given contract.
    /// This is useful when you want the item to exist, but not have a value.
    pub fn with_empty_contract_item<T>(mut self, contract: &Addr, cw_item: Item<T>) -> Self
    where
        T: Serialize + DeserializeOwned,
    {
        self.insert_contract_key_value(contract, cw_item.as_slice().to_vec(), Binary::default());

        self
    }

    /// Include a contract item in the mock querier.
    /// ```rust
    /// use cw_storage_plus::Item;
    /// use cosmwasm_std::testing::MockApi;
    /// use abstract_testing::MockQuerierBuilder;
    ///
    /// let api = MockApi::default();
    /// let contract_address = api.addr_make("contract1");
    ///
    /// const ITEM: Item<String> = Item::new("item");
    ///
    /// MockQuerierBuilder::default()
    ///     .with_contract_item(
    ///     &contract_address,
    ///     ITEM,
    ///     &"value".to_string(),
    /// );
    /// ```
    pub fn with_contract_item<T>(mut self, contract: &Addr, cw_item: Item<T>, value: &T) -> Self
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
    /// use cosmwasm_std::testing::MockApi;
    ///
    /// let api = MockApi::default();
    /// let contract_address = api.addr_make("contract1");
    ///
    /// MockQuerierBuilder::default()
    ///    .with_contract_version(&contract_address, "contract1", "v1.0.0");
    /// ```
    pub fn with_contract_version(
        self,
        contract: &Addr,
        name: impl Into<String>,
        version: impl Into<String>,
    ) -> Self {
        self.with_contract_item(
            contract,
            CONTRACT,
            &ContractVersion {
                contract: name.into(),
                version: version.into(),
            },
        )
    }
    /// set the SDK-level contract admin for a contract.
    pub fn with_contract_admin(mut self, contract: &Addr, admin: &Addr) -> Self {
        self.contract_admin.insert(contract.clone(), admin.clone());
        self
    }

    /// Build the [`MockQuerier`].
    pub fn build(mut self) -> MockQuerier {
        self.base.update_wasm(move |wasm| {
            let res = match wasm {
                WasmQuery::Raw { contract_addr, key } => {
                    let str_key = std::str::from_utf8(key.as_slice()).unwrap();
                    let addr = Addr::unchecked(contract_addr);

                    // First check for raw mappings
                    if let Some(raw_map) = self.raw_mappings.get(&addr) {
                        if let Some(value) = raw_map.get(key) {
                            return SystemResult::Ok(ContractResult::Ok(value.clone()));
                        }
                    }

                    // Then check the handlers
                    let raw_handler = self.raw_handlers.get(&addr);

                    match raw_handler {
                        Some(handler) => (*handler)(str_key),
                        None => (*self.fallback_raw_handler)(&addr, key),
                    }
                }
                WasmQuery::Smart { contract_addr, msg } => {
                    let addr = Addr::unchecked(contract_addr);
                    let contract_handler = self.smart_handlers.get(&addr);

                    match contract_handler {
                        Some(handler) => (*handler)(msg),
                        None => (*self.fallback_smart_handler)(&addr, msg),
                    }
                }
                WasmQuery::ContractInfo { contract_addr } => {
                    let addr = Addr::unchecked(contract_addr);
                    let info = ContractInfoResponse::new(
                        1,
                        Addr::unchecked(""),
                        self.contract_admin.get(&addr).map(Addr::unchecked),
                        false,
                        None,
                    );
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
    fn with_owner(self, contract: &Addr, owner: Option<&Addr>) -> Self;
}

impl MockQuerierOwnership for MockQuerierBuilder {
    fn with_owner(mut self, contract: &Addr, owner: Option<&Addr>) -> Self {
        let owner = if let Some(owner) = owner {
            GovernanceDetails::Monarchy {
                monarch: owner.clone(),
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

pub fn wrap_querier(querier: &MockQuerier) -> QuerierWrapper<'_, Empty> {
    QuerierWrapper::<Empty>::new(querier)
}

#[cfg(test)]
mod tests {
    use abstract_std::{
        account::state::{ACCOUNT_ID, ACCOUNT_MODULES},
        objects::ABSTRACT_ACCOUNT_ID,
        registry::state::ACCOUNT_ADDRESSES,
    };

    use super::*;
    use cosmwasm_std::testing::mock_dependencies;

    mod account {

        use abstract_std::registry::Account;

        use crate::abstract_mock_querier_builder;

        use super::*;

        #[test]
        fn should_return_admin_account_address() {
            let mut deps = mock_dependencies();
            deps.querier = abstract_mock_querier(deps.api);
            let abstr = AbstractMockAddrs::new(deps.api);

            let actual = ACCOUNT_ADDRESSES.query(
                &wrap_querier(&deps.querier),
                abstr.registry,
                &ABSTRACT_ACCOUNT_ID,
            );

            let expected = abstr.account;

            assert_eq!(actual, Ok(Some(expected)));
        }

        #[test]
        fn should_return_account_address() {
            let mut deps = mock_dependencies();
            let account = Account::new(deps.api.addr_make("my_account"));
            deps.querier = abstract_mock_querier_builder(deps.api)
                .account(&account, TEST_ACCOUNT_ID)
                .build();
            let abstr = AbstractMockAddrs::new(deps.api);

            let actual = ACCOUNT_ADDRESSES.query(
                &wrap_querier(&deps.querier),
                abstr.registry,
                &TEST_ACCOUNT_ID,
            );

            assert_eq!(actual, Ok(Some(account)));
        }
    }

    mod queries {
        use super::*;

        use abstract_sdk::mock_module::{MockModuleQueryMsg, MockModuleQueryResponse};
        use cosmwasm_std::QueryRequest;

        #[test]
        fn smart_query() {
            let api = MockApi::default();
            // ## ANCHOR: smart_query
            let contract_address = api.addr_make("contract_address");
            let querier = MockQuerierBuilder::default()
                .with_smart_handler(&contract_address, |msg| {
                    // handle the message
                    let MockModuleQueryMsg {} = from_json::<MockModuleQueryMsg>(msg).unwrap();
                    to_json_binary(&MockModuleQueryResponse {}).map_err(|e| e.to_string())
                })
                .build();
            // ## ANCHOR_END: smart_query

            let resp_bin = querier
                .handle_query(&QueryRequest::Wasm(WasmQuery::Smart {
                    contract_addr: contract_address.to_string(),
                    msg: to_json_binary(&MockModuleQueryMsg {}).unwrap(),
                }))
                .unwrap()
                .unwrap();
            let resp: MockModuleQueryResponse = from_json(resp_bin).unwrap();

            assert_eq!(resp, MockModuleQueryResponse {});
        }

        #[test]
        fn raw_query() {
            let api = MockApi::default();
            // ## ANCHOR: raw_query
            let contract_address = api.addr_make("contract_address");
            let querier = MockQuerierBuilder::default()
                .with_raw_handler(&contract_address, |key: &str| {
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
                    contract_addr: contract_address.to_string(),
                    key: Binary::from("the_key".joined_key()),
                }))
                .unwrap()
                .unwrap();
            let resp: String = from_json(resp_bin).unwrap();

            assert_eq!(resp, "the_value");
        }
    }

    mod account_id {
        use crate::abstract_mock_querier_builder;

        use super::*;

        #[test]
        fn should_return_admin_acct_id() {
            let mut deps = mock_dependencies();
            deps.querier = abstract_mock_querier(deps.api);
            let root_account = admin_account(deps.api);

            let actual =
                ACCOUNT_ID.query(&wrap_querier(&deps.querier), root_account.addr().clone());

            assert_eq!(actual, Ok(ABSTRACT_ACCOUNT_ID));
        }

        #[test]
        fn should_return_test_acct_id() {
            let mut deps = mock_dependencies();
            let test_base = test_account(deps.api);
            deps.querier = abstract_mock_querier_builder(deps.api)
                .account(&test_base, TEST_ACCOUNT_ID)
                .build();

            let actual = ACCOUNT_ID.query(&wrap_querier(&deps.querier), test_base.into_addr());

            assert_eq!(actual, Ok(TEST_ACCOUNT_ID));
        }
    }

    mod account_modules {
        use super::*;

        #[test]
        fn should_return_test_module_address_for_test_module() {
            let mut deps = mock_dependencies();
            deps.querier = abstract_mock_querier(deps.api);
            let abstr = AbstractMockAddrs::new(deps.api);

            let actual = ACCOUNT_MODULES.query(
                &wrap_querier(&deps.querier),
                abstr.account.into_addr(),
                TEST_MODULE_ID,
            );

            assert_eq!(actual, Ok(Some(abstr.module_address)));
        }

        // #[test]
        // fn should_return_none_for_unknown_module() {
        //     let mut deps = mock_dependencies();
        //     deps.querier = querier();
        //
        //     let actual = ACCOUNT_MODULES.query(
        //         &wrap_querier(&deps.querier),
        //         Addr::unchecked(TEST_ACCOUNT),
        //         "unknown_module",
        //     );
        //
        //     assert_that!(actual).is_ok().is_none();
        // }
    }
}
