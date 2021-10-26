// Copied from cosmwasm-std

use std::collections::HashMap;

use cosmwasm_std::testing::{MockApi, MockStorage};
use cosmwasm_std::{
    from_slice, to_binary, AllBalanceResponse, BalanceResponse, BankQuery, Binary, Coin,
    ContractResult, Empty, OwnedDeps, Querier, QuerierResult, QueryRequest, SystemError,
    SystemResult, Uint128, WasmQuery,
};

use terra_cosmwasm::SwapResponse;
use terraswap::pair::SimulationResponse;

use white_whale::denom::LUNA_DENOM;

pub const MOCK_CONTRACT_ADDR: &str = "cosmos2contract";

/// All external requirements that can be injected for unit tests.
/// It sets the given balance for the contract itself, nothing else
pub fn mock_dependencies(
    contract_balance: &[Coin],
) -> OwnedDeps<MockStorage, MockApi, MockQuerier> {
    OwnedDeps {
        storage: MockStorage::default(),
        api: MockApi::default(),
        querier: MockQuerier::new(&[(&MOCK_CONTRACT_ADDR.to_string(), contract_balance)]),
    }
}

/// MockQuerier holds an immutable table of bank balances
/// TODO: also allow querying contracts
pub struct MockQuerier {
    bank: BankQuerier,
    // placeholder to add support later
    wasm: DummyQuerier,
    /// A handler to handle custom queries. This is set to a dummy handler that
    /// always errors by default. Update it via `with_custom_handler`.
    ///
    /// Use box to avoid the need of another generic type
    // custom_handler: Box<dyn for<'a> Fn(&'a Empty) -> MockQuerierCustomHandlerResult>,
    custom: FakeMarketQuerier,
}

impl MockQuerier {
    pub fn new(balances: &[(&String, &[Coin])]) -> Self {
        MockQuerier {
            bank: BankQuerier::new(balances),
            wasm: DummyQuerier {
                pool_address: "test_pool".to_string(),
            },
            custom: FakeMarketQuerier {},
            // strange argument notation suggested as a workaround here: https://github.com/rust-lang/rust/issues/41078#issuecomment-294296365
            // custom_handler: Box::from(|_: &_| -> MockQuerierCustomHandlerResult {
            //     Ok(Ok(Binary::from(vec![0u8])))
            //     // Err(SystemError::UnsupportedRequest {
            //     //     kind: "custom".to_string(),
            //     // })
            // }),
        }
    }

    // set a new balance for the given address and return the old balance
    pub fn update_balance<U: Into<String>>(
        &mut self,
        addr: U,
        balance: Vec<Coin>,
    ) -> Option<Vec<Coin>> {
        self.bank.balances.insert(addr.into(), balance)
    }
}

impl Querier for MockQuerier {
    fn raw_query(&self, bin_request: &[u8]) -> QuerierResult {
        let request: QueryRequest<Empty> = match from_slice(bin_request) {
            Ok(v) => v,
            Err(e) => {
                return SystemResult::Err(SystemError::InvalidRequest {
                    error: format!("Parsing query request: {}", e),
                    request: bin_request.into(),
                })
            }
        };
        self.handle_query(&request)
    }
}

impl MockQuerier {
    pub fn handle_query(&self, request: &QueryRequest<Empty>) -> QuerierResult {
        match &request {
            QueryRequest::Bank(bank_query) => self.bank.query(bank_query),
            QueryRequest::Custom(custom_query) => self.custom.query(custom_query),
            QueryRequest::Wasm(msg) => self.wasm.query(msg),
            _ => SystemResult::Ok(ContractResult::Ok(to_binary("").unwrap())),
        }
    }
}

#[derive(Clone, Default)]
struct DummyQuerier {
    pool_address: String,
}

impl DummyQuerier {
    fn process_smart_query(&self, contract_addr: &str) -> QuerierResult {
        if contract_addr == self.pool_address {
            let binary_response = to_binary(&SimulationResponse {
                return_amount: Uint128::from(1000000u64),
                spread_amount: Uint128::zero(),
                commission_amount: Uint128::zero(),
            });
            if binary_response.is_err() {
                return SystemResult::Err(SystemError::Unknown {});
            }

            return SystemResult::Ok(ContractResult::Ok(binary_response.unwrap()));
        }

        SystemResult::Ok(ContractResult::Ok(Binary::from(vec![0u8])))
    }

    fn query(&self, request: &WasmQuery) -> QuerierResult {
        match request {
            WasmQuery::Smart { contract_addr, .. } => self.process_smart_query(contract_addr),
            _ => SystemResult::Ok(ContractResult::Ok(Binary::from(vec![0u8]))),
        }
    }
}

#[derive(Clone, Default)]
struct FakeMarketQuerier {}

impl FakeMarketQuerier {
    fn query(&self, _request: &Empty) -> QuerierResult {
        let binary_response = to_binary(&SwapResponse {
            receive: Coin {
                denom: LUNA_DENOM.to_string(),
                amount: Uint128::from(1000000u64),
            },
        });
        SystemResult::Ok(ContractResult::Ok(binary_response.unwrap()))
    }
}

#[derive(Clone, Default)]
pub struct BankQuerier {
    balances: HashMap<String, Vec<Coin>>,
}

impl BankQuerier {
    pub fn new(balances: &[(&String, &[Coin])]) -> Self {
        let mut map = HashMap::<String, Vec<Coin>>::new();
        for (addr, coins) in balances.iter() {
            map.insert(addr.to_string(), coins.to_vec());
        }
        BankQuerier { balances: map }
    }

    pub fn query(&self, request: &BankQuery) -> QuerierResult {
        match request {
            BankQuery::Balance { address, denom } => {
                // proper error on not found, serialize result on found
                let amount = self
                    .balances
                    .get(address)
                    .and_then(|v| v.iter().find(|c| &c.denom == denom).map(|c| c.amount))
                    .unwrap_or_default();
                let bank_res = BalanceResponse {
                    amount: Coin {
                        amount,
                        denom: denom.to_string(),
                    },
                };
                SystemResult::Ok(ContractResult::Ok(to_binary(&bank_res).unwrap()))
            }
            BankQuery::AllBalances { address } => {
                // proper error on not found, serialize result on found
                let bank_res = AllBalanceResponse {
                    amount: self.balances.get(address).cloned().unwrap_or_default(),
                };
                SystemResult::Ok(ContractResult::Ok(to_binary(&bank_res).unwrap()))
            }
            _ => SystemResult::Ok(ContractResult::Ok(Binary::from(vec![0u8]))),
        }
    }
}
