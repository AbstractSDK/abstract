# Tests covered

## Unit tests

- Contract instantiation -> src/tests/instantiate.rs
- Messages
  - ExecuteMsg::SetFee -> src/tests/msg.rs
    - unsuccessful -> unauthorized
    - successful
  - ExecuteMsg::UpdatePool -> src/tests/msg.rs
    - unsuccessful -> unauthorized
    - successful

## Integration tests
- Messages
  - ExecuteMsg::ProvideLiquidity -> src/tests/integration_tests/integration.rs
  - DepositHookMsg::WithdrawLiquidity -> src/tests/integration_tests/integration.rs
  - DepositHookMsg::ProvideLiquidity -> src/tests/integration_tests/integration.rs
  - InstantiateMsg -> -> src/tests/integration_tests/instantiate.rs
  - BaseInstantiateMsg -> -> src/tests/integration_tests/instantiate.rs
  - ExecuteMsg::UpdatePool -> -> src/tests/integration_tests/instantiate.rs
  - StateResponse -> -> src/tests/integration_tests/instantiate.rs

---

# Coverage

`commands.rs`: 94%
`contract.rs`: 83%
`error.rs`: 8%
`msg.rs`: 28%
`response.rs`: 18%
`state.rs`: 33%
