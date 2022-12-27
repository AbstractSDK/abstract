use crate::{TEST_MANAGER, TEST_MODULE_ADDRESS, TEST_MODULE_ID, TEST_PROXY, TEST_VERSION_CONTROL};

use abstract_os::version_control::Core;

use cosmwasm_std::testing::MockQuerier;
use cosmwasm_std::{
    to_binary, Addr, Binary, ContractResult, Empty, QuerierWrapper, SystemResult, WasmQuery,
};
use std::collections::HashMap;

pub type AbstractQuerier = MockQuerier<Empty>;

pub fn querier() -> AbstractQuerier {
    let mut querier = AbstractQuerier::default();
    querier.update_wasm(|wasm| {
        match wasm {
            WasmQuery::Raw { contract_addr, key } => {
                let str_key = std::str::from_utf8(&key.0).unwrap();

                let res = match contract_addr.as_str() {
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
                            Ok(to_binary(&0).unwrap())
                        } else {
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
                            Ok(Binary(vec![]))
                        }
                    }
                    _ => panic!("unexpected contract"),
                };

                match res {
                    Ok(res) => SystemResult::Ok(ContractResult::Ok(res)),
                    Err(e) => SystemResult::Ok(ContractResult::Err(e)),
                }
            }
            _ => panic!("Unexpected smart query"),
        }
    });
    querier
}

pub fn wrap_querier(querier: &AbstractQuerier) -> QuerierWrapper<'_, Empty> {
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
            deps.querier = querier();

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
            deps.querier = querier();
            let actual = OS_ID.query(&wrap_querier(&deps.querier), Addr::unchecked(TEST_MANAGER));

            assert_that!(actual).is_ok().is_equal_to(TEST_OS_ID);
        }
    }

    mod os_modules {
        use super::*;

        #[test]
        fn should_return_test_module_address_for_test_module() {
            let mut deps = mock_dependencies();
            deps.querier = querier();

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
