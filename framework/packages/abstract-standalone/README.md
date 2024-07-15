# Abstract Standalone

This crate contains a `StandaloneContract` struct that can be used to interact with the abstract on-chain infrastructure.S

## Usage

First, import the crate in your `Cargo.toml`.

```toml
[dependencies]
abstract-standalone = "<latest version>"
```

You can then define the contract's constant: 

```rust
pub const CONTRACT_ID: &str = "my-namespace:my-contract";
pub const VERSION: &str = env!("CARGO_PKG_VERSION");

pub const MY_CONTRACT: MockStandaloneContract =
        MockStandaloneContract::new(CONTRACT_ID, VERSION, None);
```

Next, include the `StandaloneInstantiateMsg` struct in your contract's instantiate message. We'll need this data to instantiate the `MY_CONTRACT` state.

```rust
use abstract_standlone::StandaloneInstantiateMsg;

#[cosmwasm_schema::cw_serde]
pub struct InstantiateMsg {
    pub base: StandaloneInstantiateMsg,
    // .. Your custom fields here.
}
```

Then, use the `StandaloneInstantiateMsg` struct to instantiate the state.

```rust
#[cosmwasm_std::entry_point]
pub fn instantiate(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
    msg: InstantiateMsg,
) -> Result<Response, MockError> {
    MY_CONTRACT.instantiate(deps, info, msg.base, true)?;

    // Your custom logic here.
}
```

Finally, you can use the `MY_CONTRACT` const to interact with the Abstract on-chain infrastructure through our [APIs](https://docs.rs/abstract-sdk/0.22.2/abstract_sdk/#traits).

```rust
#[cosmwasm_std::entry_point]
pub fn execute(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
    msg: ExecuteMsg,
) -> Result<Response, MockError> {

    // Example of using an Abstract API.
    let bank: Bank<StandaloneContract> = MY_CONTRACT.bank(deps.as_ref());
    let account_balance: Coin = bank_api.balance(&AssetEntry::new("abstract"))?;
    // ...

    Ok(MY_CONTRACT.response("mock_exec"))
}
```
