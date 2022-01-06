[# Tests covered

The `dapp-template` covers all the cases for the base operations that are shared by all dapps. Therefore, unless there
is a change in the logic, those cases are not covered in other dapps as they are covered by the template.

## Unit tests

- Contract instantiation -> src/tests/instantiate.rs
- Queries
  - BaseQueryMsg::Config -> src/tests/query.rs
- Messages
  - BaseExecuteMsg::UpdateConfig -> src/tests/msg.rs
    - unsuccessful -> unauthorized
    - successful -> with treasury_address
    - successful -> with trader
    - successful -> with memory
    - successful -> with trader, trader & memory
    - successful -> with no parameters
  - BaseExecuteMsg::SetAdmin -> src/tests/msg.rs
    - unsuccessful -> unauthorized
    - successful

---

# Coverage

`contract.rs`: 92%
]()
