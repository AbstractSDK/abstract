#[cfg(not(feature = "library"))]
use cosmwasm_std::entry_point;
use cosmwasm_std::{
    coin, ensure, to_binary, Addr, BankMsg, Binary, Coin, CosmosMsg, Decimal, Deps, DepsMut, Env,
    MessageInfo, Response, StdResult, WasmMsg,
};

use cw2::set_contract_version;
use cw20::{BalanceResponse, Cw20ExecuteMsg, Cw20QueryMsg};
use cw_utils::ensure_from_older_version;

use crate::msg::{ExecuteMsg, InstantiateMsg, MigrateMsg, QueryMsg};
use crate::state::{Config, CONFIG};
use crate::ContractError;

// version info for migration info
const CONTRACT_NAME: &str = "crates.io:cw-splitter";
const CONTRACT_VERSION: &str = env!("CARGO_PKG_VERSION");

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn instantiate(
    deps: DepsMut,
    _env: Env,
    _info: MessageInfo,
    msg: InstantiateMsg,
) -> Result<Response, ContractError> {
    set_contract_version(deps.storage, CONTRACT_NAME, CONTRACT_VERSION)?;

    let addresses = validate_addresses(deps.as_ref(), msg.addresses)?;
    let cw20_addresses = msg
        .cw20_contracts
        .into_iter()
        .map(|addr| deps.api.addr_validate(&addr))
        .collect::<StdResult<Vec<_>>>()?;

    CONFIG.save(
        deps.storage,
        &Config {
            addresses,
            cw20_addresses,
        },
    )?;

    Ok(Response::new()
        .add_attribute("action", "instnatiate")
        .add_attribute("contract", "splitter"))
}

fn validate_addresses(
    deps: Deps,
    addresses: Vec<(String, Decimal)>,
) -> Result<Vec<(Addr, Decimal)>, ContractError> {
    let mut sum = Decimal::zero();
    let addresses = addresses
        .into_iter()
        .map(|(address, weight)| {
            let address = deps.api.addr_validate(&address)?;
            sum += weight;
            Ok((address, weight))
        })
        .collect::<StdResult<Vec<(Addr, Decimal)>>>()?;

    ensure!(sum == Decimal::one(), ContractError::InvalidMsg {});
    Ok(addresses)
}

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn execute(
    deps: DepsMut,
    env: Env,
    _info: MessageInfo,
    msg: ExecuteMsg,
) -> Result<Response, ContractError> {
    match msg {
        ExecuteMsg::SendTokens {
            native_denoms,
            cw20_addresses,
        } => execute::send_tokens(deps, env, native_denoms, cw20_addresses),
    }
}

mod execute {
    use super::*;

    pub fn send_tokens(
        deps: DepsMut,
        env: Env,
        native_denoms: Vec<String>,
        cw20_addresses: Option<Vec<String>>,
    ) -> Result<Response, ContractError> {
        let config = CONFIG.load(deps.storage)?;

        let contract_address = env.contract.address.to_string();
        // gather balances of native tokens, either from function parameter or all
        let native_balances = native_denoms
            .into_iter()
            .map(|denom| deps.querier.query_balance(&env.contract.address, denom))
            .collect::<StdResult<Vec<Coin>>>()?;

        // gather addresses of cw20 token contract, either from arguments or configuration
        let cw20_addresses = if let Some(cw20_addresses) = cw20_addresses {
            cw20_addresses
                .into_iter()
                .map(|address| deps.api.addr_validate(&address))
                .collect::<StdResult<Vec<Addr>>>()?
        } else {
            config.cw20_addresses
        };

        let mut response = Response::new();

        for (address, weight) in config.addresses {
            let amount = native_balances
                .iter()
                .filter_map(|bcoin| {
                    let amount = bcoin.amount * weight;
                    if amount.is_zero() {
                        None
                    } else {
                        Some(coin((bcoin.amount * weight).u128(), &bcoin.denom))
                    }
                })
                .collect::<Vec<Coin>>();
            if !amount.is_empty() {
                let native_message = BankMsg::Send {
                    to_address: address.to_string(),
                    amount,
                };
                response = response.add_message(native_message);
            }

            let cw20_messages = cw20_addresses
                .iter()
                // filter out if balance is zero in order to avoid empty transfer error
                .filter_map(|token| {
                    match deps.querier.query_wasm_smart::<BalanceResponse>(
                        token,
                        &Cw20QueryMsg::Balance {
                            address: contract_address.clone(),
                        },
                    ) {
                        Ok(r) => {
                            if !r.balance.is_zero() {
                                Some((token, r.balance))
                            } else {
                                None
                            }
                        }
                        // the only victim of current design
                        Err(_) => None,
                    }
                })
                .map(|(token, balance)| {
                    let msg = WasmMsg::Execute {
                        contract_addr: token.to_string(),
                        msg: to_binary(&Cw20ExecuteMsg::Transfer {
                            recipient: address.to_string(),
                            amount: balance * weight,
                        })?,
                        funds: vec![],
                    }
                    .into();
                    Ok(msg)
                })
                .collect::<StdResult<Vec<CosmosMsg>>>()?;
            response = response.add_messages(cw20_messages);
        }

        Ok(response)
    }
}

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn query(deps: Deps, _env: Env, msg: QueryMsg) -> StdResult<Binary> {
    match msg {
        QueryMsg::Config {} => to_binary(&CONFIG.load(deps.storage)?),
    }
}

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn migrate(deps: DepsMut, _env: Env, msg: MigrateMsg) -> Result<Response, ContractError> {
    // needed safety check
    ensure_from_older_version(deps.storage, CONTRACT_NAME, CONTRACT_VERSION)?;

    CONFIG.remove(deps.storage);

    let addresses = validate_addresses(deps.as_ref(), msg.new_addresses)?;
    let cw20_addresses = msg
        .new_cw20_contracts
        .into_iter()
        .map(|addr| deps.api.addr_validate(&addr))
        .collect::<StdResult<Vec<_>>>()?;

    CONFIG.save(
        deps.storage,
        &Config {
            addresses,
            cw20_addresses,
        },
    )?;

    Ok(Response::new())
}

#[cfg(test)]
mod tests {
    use super::*;
    use cosmwasm_std::testing::mock_dependencies;
    use cw_multi_test::{App, ContractWrapper, Executor};

    #[test]
    fn validate_config() {
        let deps = mock_dependencies();
        let addresses = vec![("address1".to_owned(), Decimal::one())];
        assert_eq!(
            validate_addresses(deps.as_ref(), addresses).unwrap(),
            vec![(Addr::unchecked("address1".to_owned()), Decimal::one())]
        );

        let addresses = vec![
            ("address1".to_owned(), Decimal::percent(50)),
            ("address2".to_owned(), Decimal::percent(25)),
            ("address3".to_owned(), Decimal::percent(25)),
        ];
        assert_eq!(
            validate_addresses(deps.as_ref(), addresses).unwrap(),
            vec![
                (Addr::unchecked("address1".to_owned()), Decimal::percent(50)),
                (Addr::unchecked("address2".to_owned()), Decimal::percent(25)),
                (Addr::unchecked("address3".to_owned()), Decimal::percent(25))
            ]
        );

        let addresses = vec![("address1".to_owned(), Decimal::percent(101))];
        assert_eq!(
            validate_addresses(deps.as_ref(), addresses).unwrap_err(),
            ContractError::InvalidMsg {}
        );

        let addresses = vec![
            ("address1".to_owned(), Decimal::percent(50)),
            ("address2".to_owned(), Decimal::percent(25)),
            ("address3".to_owned(), Decimal::percent(26)),
        ];
        assert_eq!(
            validate_addresses(deps.as_ref(), addresses).unwrap_err(),
            ContractError::InvalidMsg {}
        );

        let addresses = vec![
            ("address1".to_owned(), Decimal::percent(50)),
            ("address2".to_owned(), Decimal::percent(25)),
            ("address3".to_owned(), Decimal::percent(24)),
        ];
        assert_eq!(
            validate_addresses(deps.as_ref(), addresses).unwrap_err(),
            ContractError::InvalidMsg {}
        );
    }

    fn store_splitter_contract(app: &mut App) -> u64 {
        let contract = Box::new(ContractWrapper::new(
            crate::contract::execute,
            crate::contract::instantiate,
            crate::contract::query,
        ));
        app.store_code(contract)
    }

    #[test]
    fn execute_with_empty_balance() {
        let mut app = App::default();

        let splitter_code_id = store_splitter_contract(&mut app);
        let splitter_contract = app
            .instantiate_contract(
                splitter_code_id,
                Addr::unchecked("owner"),
                &InstantiateMsg {
                    addresses: vec![("address1".to_owned(), Decimal::one())],
                    cw20_contracts: vec![],
                },
                &[],
                "Splitter contract",
                Some("owner".to_owned()),
            )
            .unwrap();

        // execute message with no balace on the contract
        // it will succeed, but won't do anything
        // (this tests again trying to send 0 amount)
        app.execute_contract(
            Addr::unchecked("owner"),
            splitter_contract,
            &ExecuteMsg::SendTokens {
                native_denoms: vec!["ujuno".to_owned()],
                cw20_addresses: None,
            },
            &[],
        )
        .unwrap();
    }

    #[test]
    fn split_tokens() {
        let mut app = App::new(|router, _, storage| {
            router
                .bank
                .init_balance(
                    storage,
                    &Addr::unchecked("owner"),
                    vec![coin(1_000_000, "ujuno")],
                )
                .unwrap()
        });

        let splitter_code_id = store_splitter_contract(&mut app);
        let splitter_contract = app
            .instantiate_contract(
                splitter_code_id,
                Addr::unchecked("owner"),
                &InstantiateMsg {
                    addresses: vec![
                        ("address1".to_owned(), Decimal::percent(50)),
                        ("address2".to_owned(), Decimal::percent(25)),
                        ("address3".to_owned(), Decimal::percent(25)),
                    ],
                    cw20_contracts: vec![],
                },
                &[],
                "Splitter contract",
                Some("owner".to_owned()),
            )
            .unwrap();

        // first send tokens to contract
        app.execute(
            Addr::unchecked("owner"),
            BankMsg::Send {
                to_address: splitter_contract.to_string(),
                amount: vec![coin(1_000_000, "ujuno")],
            }
            .into(),
        )
        .unwrap();

        // execute message which sends tokens according to configuration
        app.execute_contract(
            Addr::unchecked("owner"),
            splitter_contract,
            &ExecuteMsg::SendTokens {
                native_denoms: vec!["ujuno".to_owned()],
                cw20_addresses: None,
            },
            &[],
        )
        .unwrap();

        assert_eq!(
            app.wrap()
                .query_balance("address1".to_owned(), "ujuno".to_owned())
                .unwrap()
                .amount
                .u128(),
            500_000u128
        );
        assert_eq!(
            app.wrap()
                .query_balance("address2".to_owned(), "ujuno".to_owned())
                .unwrap()
                .amount
                .u128(),
            250_000u128
        );
        assert_eq!(
            app.wrap()
                .query_balance("address3".to_owned(), "ujuno".to_owned())
                .unwrap()
                .amount
                .u128(),
            250_000u128
        );
    }

    #[test]
    fn split_tokens_multiple_denoms() {
        let mut app = App::new(|router, _, storage| {
            router
                .bank
                .init_balance(
                    storage,
                    &Addr::unchecked("owner"),
                    vec![coin(1_000_000, "ujuno"), coin(200_000, "wynd")],
                )
                .unwrap()
        });

        let splitter_code_id = store_splitter_contract(&mut app);
        let splitter_contract = app
            .instantiate_contract(
                splitter_code_id,
                Addr::unchecked("owner"),
                &InstantiateMsg {
                    addresses: vec![
                        ("address1".to_owned(), Decimal::percent(33)),
                        ("address2".to_owned(), Decimal::percent(67)),
                    ],
                    cw20_contracts: vec![],
                },
                &[],
                "Splitter contract",
                Some("owner".to_owned()),
            )
            .unwrap();

        // first send tokens to contract
        app.execute(
            Addr::unchecked("owner"),
            BankMsg::Send {
                to_address: splitter_contract.to_string(),
                amount: vec![coin(1_000_000, "ujuno")],
            }
            .into(),
        )
        .unwrap();
        app.execute(
            Addr::unchecked("owner"),
            BankMsg::Send {
                to_address: splitter_contract.to_string(),
                amount: vec![coin(200_000, "wynd")],
            }
            .into(),
        )
        .unwrap();

        // execute message which sends tokens according to configuration
        app.execute_contract(
            Addr::unchecked("owner"),
            splitter_contract,
            &ExecuteMsg::SendTokens {
                native_denoms: vec!["ujuno".to_owned(), "wynd".to_owned()],
                cw20_addresses: None,
            },
            &[],
        )
        .unwrap();

        assert_eq!(
            app.wrap()
                .query_all_balances("address1".to_owned())
                .unwrap(),
            vec![coin(330_000u128, "ujuno"), coin(66_000u128, "wynd")]
        );
        assert_eq!(
            app.wrap()
                .query_all_balances("address2".to_owned())
                .unwrap(),
            vec![coin(670_000u128, "ujuno"), coin(134_000u128, "wynd")]
        );
    }

    #[test]
    fn split_tokens_specified_in_message() {
        let mut app = App::new(|router, _, storage| {
            router
                .bank
                .init_balance(
                    storage,
                    &Addr::unchecked("owner"),
                    vec![coin(1_000_000, "ujuno"), coin(200_000, "wynd")],
                )
                .unwrap()
        });

        let splitter_code_id = store_splitter_contract(&mut app);
        let splitter_contract = app
            .instantiate_contract(
                splitter_code_id,
                Addr::unchecked("owner"),
                &InstantiateMsg {
                    addresses: vec![
                        ("address1".to_owned(), Decimal::percent(33)),
                        ("address2".to_owned(), Decimal::percent(67)),
                    ],
                    cw20_contracts: vec![],
                },
                &[],
                "Splitter contract",
                Some("owner".to_owned()),
            )
            .unwrap();

        // first send tokens to contract
        app.execute(
            Addr::unchecked("owner"),
            BankMsg::Send {
                to_address: splitter_contract.to_string(),
                amount: vec![coin(1_000_000, "ujuno")],
            }
            .into(),
        )
        .unwrap();
        app.execute(
            Addr::unchecked("owner"),
            BankMsg::Send {
                to_address: splitter_contract.to_string(),
                amount: vec![coin(200_000, "wynd")],
            }
            .into(),
        )
        .unwrap();

        app.execute_contract(
            Addr::unchecked("owner"),
            splitter_contract.clone(),
            &ExecuteMsg::SendTokens {
                native_denoms: vec!["wynd".to_owned()],
                cw20_addresses: None,
            },
            &[],
        )
        .unwrap();

        assert_eq!(
            app.wrap()
                .query_all_balances("address1".to_owned())
                .unwrap(),
            vec![coin(66_000u128, "wynd")]
        );
        assert_eq!(
            app.wrap()
                .query_all_balances("address2".to_owned())
                .unwrap(),
            vec![coin(134_000u128, "wynd")]
        );
        // make sure other tokens are still on splitter contract's balance
        assert_eq!(
            app.wrap()
                .query_all_balances(splitter_contract.to_string())
                .unwrap(),
            vec![coin(1_000_000u128, "ujuno")]
        );
    }

    #[test]
    fn specify_tokens_without_balance_wont_break_contract() {
        let mut app = App::default();

        let splitter_code_id = store_splitter_contract(&mut app);
        let splitter_contract = app
            .instantiate_contract(
                splitter_code_id,
                Addr::unchecked("owner"),
                &InstantiateMsg {
                    addresses: vec![
                        ("address1".to_owned(), Decimal::percent(33)),
                        ("address2".to_owned(), Decimal::percent(67)),
                    ],
                    cw20_contracts: vec![],
                },
                &[],
                "Splitter contract",
                Some("owner".to_owned()),
            )
            .unwrap();

        // Specify tokens that splitter has no balances
        // Execute message won't fail
        app.execute_contract(
            Addr::unchecked("owner"),
            splitter_contract,
            &ExecuteMsg::SendTokens {
                native_denoms: vec!["some_token".to_owned()],
                cw20_addresses: Some(vec!["someaddress".to_owned()]),
            },
            &[],
        )
        .unwrap();
    }

    mod cw20_tests {
        use super::*;

        use cosmwasm_std::Uint128;
        use cw20::{BalanceResponse, Cw20Coin, Cw20ExecuteMsg, Cw20QueryMsg};
        use cw20_base::msg::InstantiateMsg as Cw20BaseInstantiateMsg;

        fn store_cw20(app: &mut App) -> u64 {
            let contract = Box::new(ContractWrapper::new(
                cw20_base::contract::execute,
                cw20_base::contract::instantiate,
                cw20_base::contract::query,
            ));

            app.store_code(contract)
        }

        fn init_token(
            app: &mut App,
            token_code: u64,
            name: &str,
            decimals: u8,
            owner: &str,
            init_balance: u128,
        ) -> Addr {
            app.instantiate_contract(
                token_code,
                Addr::unchecked(owner),
                &Cw20BaseInstantiateMsg {
                    symbol: name.to_owned(),
                    name: name.to_owned(),
                    decimals,
                    initial_balances: vec![Cw20Coin {
                        address: owner.into(),
                        amount: Uint128::from(init_balance),
                    }],
                    mint: None,
                    marketing: None,
                },
                &[],
                "{name}_token",
                None,
            )
            .unwrap()
        }

        pub fn token_balance(
            app: &App,
            token_addr: impl Into<String>,
            user: impl Into<String>,
        ) -> u128 {
            let resp: BalanceResponse = app
                .wrap()
                .query_wasm_smart(
                    token_addr,
                    &Cw20QueryMsg::Balance {
                        address: user.into(),
                    },
                )
                .unwrap();

            resp.balance.u128()
        }

        #[test]
        fn execute_with_empty_balance() {
            let mut app = App::default();

            let token_code_id = store_cw20(&mut app);
            let token_contract =
                init_token(&mut app, token_code_id, "TOKEN", 9, "owner", 1_000_000);
            let token_contract2 =
                init_token(&mut app, token_code_id, "TTOKEN", 9, "owner", 1_000_000);

            let splitter_code_id = store_splitter_contract(&mut app);
            let splitter_contract = app
                .instantiate_contract(
                    splitter_code_id,
                    Addr::unchecked("owner"),
                    &InstantiateMsg {
                        addresses: vec![("address1".to_owned(), Decimal::one())],
                        cw20_contracts: vec![
                            token_contract.to_string(),
                            token_contract2.to_string(),
                        ],
                    },
                    &[],
                    "Splitter contract",
                    Some("owner".to_owned()),
                )
                .unwrap();

            // execute message with no balace on the contract
            // it will succeed, but won't do anything
            // (this tests again trying to send 0 amount)
            app.execute_contract(
                Addr::unchecked("owner"),
                splitter_contract.clone(),
                &ExecuteMsg::SendTokens {
                    native_denoms: vec![],
                    cw20_addresses: None,
                },
                &[],
            )
            .unwrap();

            // now send tokens but only of one of specified cw20 tokens
            app.execute(
                Addr::unchecked("owner"),
                WasmMsg::Execute {
                    contract_addr: token_contract2.to_string(),
                    msg: to_binary(&Cw20ExecuteMsg::Transfer {
                        recipient: splitter_contract.to_string(),
                        amount: 1_000_000u128.into(),
                    })
                    .unwrap(),
                    funds: vec![],
                }
                .into(),
            )
            .unwrap();

            app.execute_contract(
                Addr::unchecked("owner"),
                splitter_contract,
                &ExecuteMsg::SendTokens {
                    native_denoms: vec![],
                    cw20_addresses: None,
                },
                &[],
            )
            .unwrap();
            assert_eq!(
                token_balance(&app, &token_contract2, "address1"),
                1_000_000u128
            );
        }

        #[test]
        fn split_tokens() {
            let mut app = App::default();

            let token_code_id = store_cw20(&mut app);
            let token_contract =
                init_token(&mut app, token_code_id, "TOKEN", 9, "owner", 1_000_000);

            let splitter_code_id = store_splitter_contract(&mut app);
            let splitter_contract = app
                .instantiate_contract(
                    splitter_code_id,
                    Addr::unchecked("owner"),
                    &InstantiateMsg {
                        addresses: vec![
                            ("address1".to_owned(), Decimal::percent(50)),
                            ("address2".to_owned(), Decimal::percent(25)),
                            ("address3".to_owned(), Decimal::percent(25)),
                        ],
                        cw20_contracts: vec![token_contract.to_string()],
                    },
                    &[],
                    "Splitter contract",
                    Some("owner".to_owned()),
                )
                .unwrap();

            // first send tokens to contract
            app.execute(
                Addr::unchecked("owner"),
                WasmMsg::Execute {
                    contract_addr: token_contract.to_string(),
                    msg: to_binary(&Cw20ExecuteMsg::Transfer {
                        recipient: splitter_contract.to_string(),
                        amount: 1_000_000u128.into(),
                    })
                    .unwrap(),
                    funds: vec![],
                }
                .into(),
            )
            .unwrap();

            // execute message which sends tokens according to configuration
            app.execute_contract(
                Addr::unchecked("owner"),
                splitter_contract,
                &ExecuteMsg::SendTokens {
                    native_denoms: vec![],
                    cw20_addresses: None,
                },
                &[],
            )
            .unwrap();

            assert_eq!(
                token_balance(&app, &token_contract, "address1"),
                500_000u128
            );
            assert_eq!(
                token_balance(&app, &token_contract, "address2"),
                250_000u128
            );
            assert_eq!(
                token_balance(&app, &token_contract, "address2"),
                250_000u128
            );
        }

        #[test]
        fn split_tokens_multiple_denoms() {
            let mut app = App::new(|router, _, storage| {
                router
                    .bank
                    .init_balance(
                        storage,
                        &Addr::unchecked("owner"),
                        vec![coin(3_000_000, "ujuno")],
                    )
                    .unwrap()
            });

            let token_code_id = store_cw20(&mut app);
            let token_contract =
                init_token(&mut app, token_code_id, "TOKEN", 9, "owner", 1_000_000);
            let token_contract2 =
                init_token(&mut app, token_code_id, "TTOKEN", 9, "owner", 2_000_000);

            let splitter_code_id = store_splitter_contract(&mut app);
            let splitter_contract = app
                .instantiate_contract(
                    splitter_code_id,
                    Addr::unchecked("owner"),
                    &InstantiateMsg {
                        addresses: vec![
                            ("address1".to_owned(), Decimal::percent(30)),
                            ("address2".to_owned(), Decimal::percent(70)),
                        ],
                        cw20_contracts: vec![
                            token_contract.to_string(),
                            token_contract2.to_string(),
                        ],
                    },
                    &[],
                    "Splitter contract",
                    Some("owner".to_owned()),
                )
                .unwrap();

            // first send tokens to contract
            app.execute(
                Addr::unchecked("owner"),
                BankMsg::Send {
                    to_address: splitter_contract.to_string(),
                    amount: vec![coin(3_000_000, "ujuno")],
                }
                .into(),
            )
            .unwrap();
            app.execute(
                Addr::unchecked("owner"),
                WasmMsg::Execute {
                    contract_addr: token_contract.to_string(),
                    msg: to_binary(&Cw20ExecuteMsg::Transfer {
                        recipient: splitter_contract.to_string(),
                        amount: 1_000_000u128.into(),
                    })
                    .unwrap(),
                    funds: vec![],
                }
                .into(),
            )
            .unwrap();
            app.execute(
                Addr::unchecked("owner"),
                WasmMsg::Execute {
                    contract_addr: token_contract2.to_string(),
                    msg: to_binary(&Cw20ExecuteMsg::Transfer {
                        recipient: splitter_contract.to_string(),
                        amount: 2_000_000u128.into(),
                    })
                    .unwrap(),
                    funds: vec![],
                }
                .into(),
            )
            .unwrap();

            // execute message which sends tokens according to configuration
            app.execute_contract(
                Addr::unchecked("owner"),
                splitter_contract,
                &ExecuteMsg::SendTokens {
                    native_denoms: vec!["ujuno".to_owned()],
                    cw20_addresses: None,
                },
                &[],
            )
            .unwrap();

            assert_eq!(
                app.wrap()
                    .query_all_balances("address1".to_owned())
                    .unwrap(),
                vec![coin(900_000u128, "ujuno")]
            );
            assert_eq!(
                token_balance(&app, &token_contract, "address1"),
                300_000u128
            );
            assert_eq!(
                token_balance(&app, &token_contract2, "address1"),
                600_000u128
            );

            assert_eq!(
                app.wrap()
                    .query_all_balances("address2".to_owned())
                    .unwrap(),
                vec![coin(2_100_000u128, "ujuno")]
            );
            assert_eq!(
                token_balance(&app, &token_contract, "address2"),
                700_000u128
            );
            assert_eq!(
                token_balance(&app, &token_contract2, "address2"),
                1_400_000u128
            );
        }

        #[test]
        fn split_tokens_specified_in_message() {
            let mut app = App::default();

            let token_code_id = store_cw20(&mut app);
            let token_contract =
                init_token(&mut app, token_code_id, "TOKEN", 9, "owner", 1_000_000);
            let token_contract2 =
                init_token(&mut app, token_code_id, "TTOKEN", 9, "owner", 2_000_000);

            let splitter_code_id = store_splitter_contract(&mut app);
            let splitter_contract = app
                .instantiate_contract(
                    splitter_code_id,
                    Addr::unchecked("owner"),
                    &InstantiateMsg {
                        addresses: vec![
                            ("address1".to_owned(), Decimal::percent(30)),
                            ("address2".to_owned(), Decimal::percent(70)),
                        ],
                        cw20_contracts: vec![
                            token_contract.to_string(),
                            token_contract2.to_string(),
                        ],
                    },
                    &[],
                    "Splitter contract",
                    Some("owner".to_owned()),
                )
                .unwrap();

            // first send tokens to contract
            app.execute(
                Addr::unchecked("owner"),
                WasmMsg::Execute {
                    contract_addr: token_contract.to_string(),
                    msg: to_binary(&Cw20ExecuteMsg::Transfer {
                        recipient: splitter_contract.to_string(),
                        amount: 1_000_000u128.into(),
                    })
                    .unwrap(),
                    funds: vec![],
                }
                .into(),
            )
            .unwrap();
            app.execute(
                Addr::unchecked("owner"),
                WasmMsg::Execute {
                    contract_addr: token_contract2.to_string(),
                    msg: to_binary(&Cw20ExecuteMsg::Transfer {
                        recipient: splitter_contract.to_string(),
                        amount: 2_000_000u128.into(),
                    })
                    .unwrap(),
                    funds: vec![],
                }
                .into(),
            )
            .unwrap();

            // Specify only one cw20 denom to split from
            app.execute_contract(
                Addr::unchecked("owner"),
                splitter_contract.clone(),
                &ExecuteMsg::SendTokens {
                    native_denoms: vec![],
                    cw20_addresses: Some(vec![token_contract2.to_string()]),
                },
                &[],
            )
            .unwrap();

            assert_eq!(token_balance(&app, &token_contract, "address1"), 0u128);
            assert_eq!(
                token_balance(&app, &token_contract2, "address1"),
                600_000u128
            );

            assert_eq!(token_balance(&app, &token_contract, "address2"), 0u128);
            assert_eq!(
                token_balance(&app, &token_contract2, "address2"),
                1_400_000u128
            );

            // make sure other cw20 tokens are still assigned to splitter's contract
            assert_eq!(
                token_balance(&app, &token_contract, splitter_contract),
                1_000_000u128
            );
        }
    }
}
