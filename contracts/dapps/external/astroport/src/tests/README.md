# Tests covered

## Unit tests

- Messages
  - ExecuteMsg::ProvideLiquidity -> src/tests/msg.rs
    - unsuccessful -> unauthorized
    - unsuccessful -> nonexisting asset

## Integration tests

- Messages
  - ExecuteMsg::ProvideLiquidity -> src/tests/integration_tests/integration.rs
  - ExecuteMsg::DetailedProvideLiquidity -> src/tests/integration_tests/integration.rs
  - ExecuteMsg::WithdrawLiquidity -> src/tests/integration_tests/integration.rs
  - ExecuteMsg::SwapAsset -> src/tests/integration_tests/integration.rs

---

# Coverage

`astroport_msg.rs`: 74%
`commands.rs`: 87%
`contract.rs`: 77%
`error.rs`: 12%
`msg.rs`: 5%
`utils.rs`: 71%
