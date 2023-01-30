use crate::{
    TEST_MANAGER, TEST_MODULE_ADDRESS, TEST_MODULE_ID, TEST_MODULE_RESPONSE, TEST_OS_ID,
    TEST_PROXY, TEST_VERSION_CONTROL,
};
use abstract_os::version_control::Core;
use cosmwasm_std::testing::MockQuerier;
use cosmwasm_std::{
    from_binary, to_binary, Addr, Binary, ContractResult, Empty, QuerierWrapper, SystemResult,
    WasmQuery,
};
use std::collections::HashMap;

pub type EmptyMockQuerier = MockQuerier<Empty>;

type BinaryQueryResult = Result<Binary, String>;
type ContractAddr = String;
type FallbackHandler = dyn for<'a> Fn(&'a str, &'a Binary) -> BinaryQueryResult;
type SmartHandler = dyn for<'a> Fn(&'a Binary) -> BinaryQueryResult;
type RawHandler = dyn for<'a> Fn(&'a str) -> BinaryQueryResult;

/// [`MockQuerierBuilder`] is a helper to build a [`MockQuerier`].
pub struct MockQuerierBuilder {
    base: EmptyMockQuerier,
    fallback_raw_handler: Box<FallbackHandler>,
    fallback_smart_handler: Box<FallbackHandler>,
    smart_handlers: HashMap<ContractAddr, Box<SmartHandler>>,
    raw_handlers: HashMap<ContractAddr, Box<RawHandler>>,
}

impl Default for MockQuerierBuilder {
    /// Create a default
    fn default() -> Self {
        let raw_fallback: fn(&str, &Binary) -> BinaryQueryResult =
            |_, _| panic!("No mock querier for this query");
        let smart_fallback: fn(&str, &Binary) -> BinaryQueryResult =
            |_, _| Err("unexpected contract".into());

        Self {
            base: MockQuerier::default(),
            fallback_raw_handler: Box::from(raw_fallback),
            fallback_smart_handler: Box::from(smart_fallback),
            smart_handlers: HashMap::default(),
            raw_handlers: HashMap::default(),
        }
    }
}

/// Helper to build a MockQuerier.
/// Usage:
/// ```rust
/// use cosmwasm_std::{from_binary, to_binary};
/// use abstract_testing::MockQuerierBuilder;
/// use cosmwasm_std::testing::MockQuerier;
/// use abstract_testing::mock_module::MockModuleExecuteMsg;
///
/// let querier = MockQuerierBuilder::default().with_smart_handler("contract_address", |msg| {
///    // handle the message
///     let res = match from_binary::<MockModuleExecuteMsg>(msg).unwrap() {
///         // handle the message
///        _ => panic!("unexpected message"),
///    };
///
///   Ok(to_binary(&msg).unwrap())
/// }).build();
/// ```
impl MockQuerierBuilder {
    pub fn with_fallback_smart_handler<SH: 'static>(mut self, handler: SH) -> Self
    where
        SH: Fn(&str, &Binary) -> BinaryQueryResult,
    {
        self.fallback_smart_handler = Box::new(handler);
        self
    }

    pub fn with_fallback_raw_handler<RH: 'static>(mut self, handler: RH) -> Self
    where
        RH: Fn(&str, &Binary) -> BinaryQueryResult,
    {
        self.fallback_raw_handler = Box::new(handler);
        self
    }

    /// Add a smart contract handler to the mock querier. The handler will be called when the
    /// contract address is queried with the given message.
    /// Usage:
    /// ```rust
    /// use cosmwasm_std::{from_binary, to_binary};
    /// use abstract_testing::MockQuerierBuilder;
    /// use cosmwasm_std::testing::MockQuerier;
    /// use abstract_testing::mock_module::MockModuleExecuteMsg;
    ///
    /// let querier = MockQuerierBuilder::default().with_smart_handler("contract_address", |msg| {
    ///    // handle the message
    ///     let res = match from_binary::<MockModuleExecuteMsg>(msg).unwrap() {
    ///         // handle the message
    ///        _ => panic!("unexpected message"),
    ///    };
    ///
    ///   Ok(to_binary(&msg).unwrap())
    /// }).build();
    /// ```
    pub fn with_smart_handler<SH: 'static>(mut self, contract: &str, handler: SH) -> Self
    where
        SH: Fn(&Binary) -> BinaryQueryResult,
    {
        self.smart_handlers
            .insert(contract.to_string(), Box::new(handler));
        self
    }

    pub fn with_raw_handler<RH: 'static>(mut self, contract: &str, handler: RH) -> Self
    where
        RH: Fn(&str) -> BinaryQueryResult,
    {
        self.raw_handlers
            .insert(contract.to_string(), Box::new(handler));
        self
    }

    /// Build the [`MockQuerier`].
    pub fn build(mut self) -> EmptyMockQuerier {
        self.base.update_wasm(move |wasm| {
            let res = match wasm {
                WasmQuery::Raw { contract_addr, key } => {
                    let str_key = std::str::from_utf8(&key.0).unwrap();

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
                // TODO: contract info
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

/// A mock querier that returns the following responses for the following **RAW** contract -> queries:
/// - TEST_PROXY
///   - "admin" -> TEST_MANAGER
/// - TEST_MANAGER
///   - "os_modules:TEST_MODULE_ID" -> TEST_MODULE_ADDRESS
///   - "os_id" -> TEST_OS_ID
/// - TEST_VERSION_CONTROL
///   - "os_core" -> { TEST_PROXY, TEST_MANAGER }
pub fn mock_querier() -> EmptyMockQuerier {
    let raw_handler = |contract: &str, key: &Binary| {
        let str_key = std::str::from_utf8(&key.0).unwrap();
        match contract {
            TEST_PROXY => match str_key {
                "admin" => Ok(to_binary(&TEST_MANAGER).unwrap()),
                _ => Err("unexpected key".to_string()),
            },
            TEST_MANAGER => {
                // add module
                let map_key = map_key("os_modules", TEST_MODULE_ID);
                let mut modules = HashMap::<Binary, Addr>::default();
                modules.insert(
                    Binary(map_key.as_bytes().to_vec()),
                    Addr::unchecked(TEST_MODULE_ADDRESS),
                );

                if let Some(value) = modules.get(key) {
                    Ok(to_binary(&value.clone()).unwrap())
                } else if str_key == "\u{0}{5}os_id" {
                    Ok(to_binary(&TEST_OS_ID).unwrap())
                } else {
                    // Return the default value
                    Ok(Binary(vec![]))
                }
            }
            TEST_VERSION_CONTROL => {
                if str_key == "\0\u{7}os_core\0\0\0\0" {
                    Ok(to_binary(&Core {
                        manager: Addr::unchecked(TEST_MANAGER),
                        proxy: Addr::unchecked(TEST_PROXY),
                    })
                    .unwrap())
                } else {
                    // Default value
                    Ok(Binary(vec![]))
                }
            }
            _ => Err("unexpected contract".to_string()),
        }
    };

    MockQuerierBuilder::default()
        .with_fallback_raw_handler(raw_handler)
        .with_smart_handler(TEST_MODULE_ADDRESS, |msg| {
            let Empty {} = from_binary(msg).unwrap();
            Ok(to_binary(TEST_MODULE_RESPONSE).unwrap())
        })
        .build()
}

pub fn wrap_querier(querier: &EmptyMockQuerier) -> QuerierWrapper<'_, Empty> {
    QuerierWrapper::<Empty>::new(querier)
}

// TODO: Fix to actually make this work!
fn map_key<'a>(namespace: &'a str, key: &'a str) -> String {
    let line_feed_char = b"\x0a";
    let mut res = vec![0u8];
    res.extend_from_slice(line_feed_char);
    res.extend_from_slice(namespace.as_bytes());
    res.extend_from_slice(key.as_bytes());
    std::str::from_utf8(&res).unwrap().to_string()
}

#[cfg(test)]
mod tests {
    use crate::{TEST_MODULE_ID, TEST_OS_ID};

    use super::*;
    use abstract_os::manager::state::OS_MODULES;
    use abstract_os::proxy::state::OS_ID;
    use abstract_os::version_control::state::OS_ADDRESSES;
    use cosmwasm_std::testing::mock_dependencies;
    use speculoos::prelude::*;

    mod os_core {
        use super::*;

        #[test]
        fn should_return_os_address() {
            let mut deps = mock_dependencies();
            deps.querier = mock_querier();

            let actual = OS_ADDRESSES.query(
                &wrap_querier(&deps.querier),
                Addr::unchecked(TEST_VERSION_CONTROL),
                TEST_OS_ID,
            );

            let expected = Core {
                proxy: Addr::unchecked(TEST_PROXY),
                manager: Addr::unchecked(TEST_MANAGER),
            };

            assert_that!(actual).is_ok().is_some().is_equal_to(expected)
        }
    }

    mod os_id {
        use super::*;

        #[test]
        fn should_return_test_os_id_with_test_manager() {
            let mut deps = mock_dependencies();
            deps.querier = mock_querier();
            let actual = OS_ID.query(&wrap_querier(&deps.querier), Addr::unchecked(TEST_MANAGER));

            assert_that!(actual).is_ok().is_equal_to(TEST_OS_ID);
        }
    }

    mod os_modules {
        use super::*;

        #[test]
        fn should_return_test_module_address_for_test_module() {
            let mut deps = mock_dependencies();
            deps.querier = mock_querier();

            let actual = OS_MODULES.query(
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
        //     let actual = OS_MODULES.query(
        //         &wrap_querier(&deps.querier),
        //         Addr::unchecked(TEST_MANAGER),
        //         "unknown_module",
        //     );
        //
        //     assert_that!(actual).is_ok().is_none();
        // }
    }
}
