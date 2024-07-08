# CW Ownable

Utility for controlling ownership of [CosmWasm](https://github.com/CosmWasm/cosmwasm) smart contracts.

## How to use

Initialize the owner during instantiation using the `initialize_owner` method provided by this crate:

```rust
use cosmwasm_std::{entry_point, DepsMut, Env, MessageInfo, Response};
use cw_gov_ownable::OwnershipError;

#[entry_point]
pub fn instantiate(
    deps: DepsMut,
    env: Env,
    _info: MessageInfo,
    msg: InstantiateMsg,
) -> Result<Response<Empty>, OwnershipError> {
    cw_gov_ownable::initialize_owner(deps.storage, deps.api, msg.owner.as_deref())?;
    Ok(Response::new())
}
```

Use the `#[cw_ownable_execute]` macro to extend your execute message:

```rust
use cosmwasm_schema::cw_serde;
use cw_gov_ownable::cw_ownable_execute;

#[cw_ownable_execute]
#[cw_serde]
enum ExecuteMsg {
    Foo {},
    Bar {},
}
```

The macro inserts a new variant, `UpdateOwnership` to the enum:

```rust
#[cw_serde]
enum ExecuteMsg {
    UpdateOwnership(cw_gov_ownable::Action),
    Foo {},
    Bar {},
}
```

Where `Action` can be one of three:

- Propose to transfer the contract's ownership to another account
- Accept the proposed ownership transfer
- Renounce the ownership, permanently setting the contract's owner to vacant

Handle the messages using the `update_ownership` function provided by this crate:

```rust
use cosmwasm_std::{entry_point, DepsMut, Env, MessageInfo, Response};
use cw_gov_ownable::{cw_serde, update_ownership, OwnershipError};

#[entry_point]
pub fn execute(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
    msg: ExecuteMsg,
) -> Result<Response, OwnershipError> {
    match msg {
        ExecuteMsg::UpdateOwnership(action) => {
            update_ownership(deps, &env.block, &info.sender, action)?;
        }
        _ => unimplemneted!(),
    }
    Ok(Response::new())
}
```

Use the `#[cw_ownable_query]` macro to extend your query message:

```rust
use cosmwasm_schema::{cw_serde, QueryResponses};
use cw_gov_ownable::cw_ownable_query;

#[cw_ownable_query]
#[cw_serde]
#[derive(QueryResponses)]
pub enum QueryMsg {
    #[returns(FooResponse)]
    Foo {},
    #[returns(BarResponse)]
    Bar {},
}
```

The macro inserts a new variant, `Ownership`:

```rust
#[cw_serde]
#[derive(QueryResponses)]
enum QueryMsg {
    #[returns(Ownership<String>)]
    Ownership {},
    #[returns(FooResponse)]
    Foo {},
    #[returns(BarResponse)]
    Bar {},
}
```

Handle the message using the `get_ownership` function provided by this crate:

```rust
use cosmwasm_std::{entry_point, Deps, Env, Binary};
use cw_gov_ownable::get_ownership;

#[entry_point]
pub fn query(deps: Deps, env: Env, msg: QueryMsg) -> StdResult<Binary> {
    match msg {
        QueryMsg::Ownership {} => to_binary(&get_ownership(deps.storage)?),
        _ => unimplemented!(),
    }
}
```

## License

Contents of this crate at or prior to version `0.5.0` are published under [GNU Affero General Public License v3](https://github.com/steak-enjoyers/cw-plus-plus/blob/9c8fcf1c95b74dd415caf5602068c558e9d16ecc/LICENSE) or later; contents after the said version are published under [Apache-2.0](../../LICENSE) license.
