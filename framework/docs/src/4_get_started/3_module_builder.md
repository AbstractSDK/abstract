# Module Builder

Abstract provides multiple module bases, as detailed in our section on [modules](../3_framework/6_module_types.md).
These bases (`App` and `Adapter`) implement some basic functionality and store some abstract-related state that will enable you to easily interact with our infrastructure through our SDK (which we'll introduce later).

For now just know that we provide you with a builder pattern that allows you to easily add custom logic to these module bases. In the rest of this section we'll outline how you can use this builder pattern to add custom functionality to your contract.

## Overview

The builder pattern employed in building an Abstract module is a slight variation of the actual "builder" design pattern. Instead of creating a new builder at runtime, our module builder lets you set custom attributes on your module at compile time, meaning you end up with a `const` value can be heavily optimized by the compiler. This system ensures that the overhead of using Abstract has little effect on both the code's runtime and WASM binary size.

```admonish info
The code-snippets in this example can be found in the <a href="https://github.com/AbstractSDK/app-template" target="_blank">app template</a>.
```

In this tutorial we will be working on an [`App` module](../3_framework/6_module_types.md#apps).

### App Type

Your custom `AppType` will be a type alias for a specific type that fills in the base `AppContract` type provided by the `abstract-app` crate. By constructing this type you're defining which messages you expect to receive at the custom endpoints of your contract.

Here's what this looks like in the template:

```rust,ignore
// src/contract.rs
pub type App = AppContract<AppError, AppInstantiateMsg, AppExecuteMsg, AppQueryMsg, AppMigrateMsg>;
```

The type above contains all the mandatory types (`Error`, `Instantiate`, `Execute`, `Query`). An optional `MigrateMsg` type is also added to allow you to customize the migration logic of your contract.

This new `App` type alias will be used in a few more places throughout the contract, so it's a good idea to define it at the top of the file.

### Module ID

The Module identifier (Module ID) is a string that will identify your application. We covered it in detail in the section on modules, [here](../3_framework/6_module_types.md#module-ids).

You define your ID as a `&'static str` like so:

```rust,ignore
pub const APP_ID: &str = "my-namespace:app";
```

### Module Version

This is the version of your module. The version will be stored on-chain. When installing a module that depends on your module, our infrastructure will assert its version requirements. Ensuring that the contracts that depend on each other are version compatible. We'll cover dependencies in more detail in the [dependencies section](./8_dependencies.md).

```rust,ignore
pub const APP_VERSION: &str = env!("CARGO_PKG_VERSION");
```

By default you should use the version of your package as your app version. That is what the `env!` macro is doing in the example above. Alternatively you can provide any 3-digit version number as a valid version.

## Build The App

Now that you have defined your type and all your attributes you can begin using the builder. To initiate this, first create the builder for the App:

```rust ignore
// src/contract.rs
const APP: App = App::new(APP_ID, APP_VERSION, None)
```

The builder constructor takes three variables:

1. `module_id`: The module ID is a string that we defined above.
2. `contract_version`: The contract version.
3. `metadata`: An optional URL that can be used to retrieve data off-chain. Can be used with the Abstract Metadata Standard to automatically generate interactive front-end components for the module. This is explained in more detail in the [metadata section](./9_metadata.md).

Amazing! You now have a very basic Abstract module. You can now add your custom logic to your module by adding handlers to the module.

Below we've defined a complete `App` module with a few custom handlers set:

```rust ignore
const APP: App = App::new(APP_ID, APP_VERSION, None)
    .with_instantiate(handlers::instantiate_handler)
    .with_execute(handlers::execute_handler)
    .with_query(handlers::query_handler)
    .with_migrate(handlers::migrate_handler)
    .with_replies(&[(INSTANTIATE_REPLY_ID, replies::instantiate_reply)]);
```

### Handlers

> The handler functions are defined in the `src/handlers` dir.

The app can then be customized by adding handler functions for your endpoints. These functions are executed whenever a specific endpoint is called on the module.

#### Writing a handler function

These handlers are where you will write your custom logic for your App. For example, below we've defined a custom `execute` handler that handles all the different `AppExecuteMsg` variants of our module.

A special feature of these functions is that we insert the instance of your module into the function's attributes. This enables you to access the module struct in your code. You will learn why this is such a powerful feature in the next section on the [Abstract SDK](./4_sdk.md).

```rust ignore
// src/handlers/execute.rs
pub fn execute_handler(
    deps: DepsMut,
    _env: Env,
    info: MessageInfo,
    app: App,
    msg: AppExecuteMsg,
) -> AppResult {
    match msg {
        AppExecuteMsg::Increment {} => increment(deps, app),
        AppExecuteMsg::Reset { count } => reset(deps, info, count, app),
        AppExecuteMsg::UpdateConfig {} => update_config(deps, info, app),
    }
}
```

The code above should look very familiar. It's only a slight variation of the code you would write in a regular CosmWasm contract. The only difference is that you have access to the `app: App` attribute, which is the instance of your module.

```admonish info
You can find more application code to read in our <a href="https://github.com/AbstractSDK/awesome-abstract" target="_blank">💥 Awesome Abstract repository 💥</a>.
```

## Summary

The Abstract SDK allows you to easily make new custom smart contracts through a simple builder pattern and straight forward type system usage.

In the next section we'll cover how you can use the module object that we make available in the function handlers to write highly functional smart contract code.

Ever wanted to swap on any cosmos DEX with only one line of code? Look no further!

## Appendix

This appendix contains all the available handlers, what type of handler `Fn` they expect and the format of the messages that are exposed on the contract endpoints.

**An overview of the available handlers:**

- `with_execute`: Called when the App's `ExecuteMsg` is called on the instantiate entry point.
- `with_instantiate`: Called when the App's `InstantiateMsg` is called on the instantiate entry point.
- `with_query`: Called when the App's `QueryMsg::Module` is called on the query entry point.
- `with_migrate`: Called when the App's `MigrateMsg` is called on the migrate entry point.
- `with_replies`: Called when the App's reply entry point is called. Matches the function's associated reply-id.
- `with_sudo`: Called when the App's `SudoMsg` is called on the sudo entry point.
- `with_receive`: Called when the App's `ExecuteMsg::Receive` variant is called on the execute entry point.
- `with_ibc_callbacks`: Called when the App's `ExecuteMsg::IbcCallback` is called on the execute entry point. Matches the callback's callback ID to its associated function.

In the case of adapters, the handlers are the same, except for `with_migrate` and `with_sudo` that are missing for reasons we explain in the [adapter section](../3_framework/6_module_types.md#adapters).

For a full overview of the list of handlers available, please refer to the respective module type documentation:

- <a href="https://docs.rs/abstract-app/latest/abstract_app/state/struct.AppContract.html" target="_blank">App</a>
- <a href="https://docs.rs/abstract-adapter/latest/abstract_adapter/state/struct.AdapterContract.html" target="_blank">
  Adapter</a>

Below, we examine each handler in greater detail. The `base` fields and variants mentioned in the messages below are defined by [the base module type](../3_framework/6_module_types.md) that you chose to use, an `App` in this case.

### Instantiate

The instantiate entry point is a mutable entry point of the contract that can only be called on contract instantiation.
Instantiation of a contract is essentially the association of a public address to a contract's state.

<details>

#### Function Signature

Expected function signature for the custom instantiate handler:

```rust,ignore
{{#include ../../../packages/abstract-sdk/src/base/contract_base.rs:init}}
```

#### Message

In order to instantiate an Abstract Module, you need to provide an InstantiateMsg with the following structure:

```rust,ignore
{{#include ../../../packages/abstract-core/src/base.rs:init}}
```

When the module's instantiate function is called the struct's `module` field is passed to your custom instantiation
handler for you to perform any custom logic.
</details>

### Execute

The execute entry point is a mutable entry point of the contract. Logic in this function can update the contract's state
and trigger state changes in other contracts by calling them. It is where the majority of your contract's logic will
reside.

<details>

#### Function Signature

Expected function signature for the custom execute handler:

```rust,ignore
{{#include ../../../packages/abstract-sdk/src/base/contract_base.rs:exec}}
```

#### Message

Called when the App's `ExecuteMsg::Module` variant is called on the execute entry point.

```rust,ignore
{{#include ../../../packages/abstract-core/src/base.rs:exec}}
```

The content of the `Module` variant is passed to your custom execute handler.
</details>

### Query

The query entry point is the non-mutable entry point of the contract. Like its name implies it it used to retrieve data
from the contract's state. This state retrieval can have a computation component but it can not alter the contract's or
any other state.

<details>

#### Function Signature

Expected function signature for the custom query handler:

```rust,ignore
{{#include ../../../packages/abstract-sdk/src/base/contract_base.rs:query}}
```

#### Message

Called when the App's `QueryMsg::Module` variant is called on the query entry point.

```rust,ignore
{{#include ../../../packages/abstract-core/src/base.rs:query}}
```

The content of the `Module` variant is passed to your custom query handler.
</details>

### Migrate

The migrate entry point is a mutable entry point that is called **after** a code_id change is applied to the contract. A
migration in CosmWasm essentially swaps out the code that's executed at the contract's address while keeping the state
as-is. The implementation of this function is often used to change the format of the contract's state by loading the
data as the original format and overwriting it with a new format, in case it changed. All adapter base implementations
already perform version assertions that make it impossible to migrate to a contract with a different ID or with a
version that is lesser or equal to the old version.

<details>

#### Function Signature

Expected function signature for the custom migrate handler:

```rust,ignore
{{#include ../../../packages/abstract-sdk/src/base/contract_base.rs:mig}}
```

#### Message

Called when the App's migrate entry point is called. Uses the struct's `module` field to customize the migration. Only
this field is passed to the handler function.

```rust,ignore
{{#include ../../../packages/abstract-core/src/base.rs:migrate}}
```

</details>

### Reply

The reply entry point is a mutable entry point that is optionally called **after** a previous mutable action. It is
often used by factory contracts to retrieve the contract of a newly instantiated contract. It essentially provides the
ability perform callbacks on actions. A reply can be requested using CosmWasm's `SubMsg` type and requires a
unique `ReplyId` which is a `u64`. The customizable handler takes an array of `(ReplyId, ReplyFn)` tuples and matches
any incoming reply on the correct `ReplyId` for you.

<details>

#### Function Signature

Expected function signature for the custom reply handler:

```rust,ignore
{{#include ../../../packages/abstract-sdk/src/base/contract_base.rs:reply}}
```

#### Message

There is no customizable message associated with this entry point.

</details>

### Sudo

The sudo entry point is a mutable entry point that can only be called by the chain's governance module. I.e. any calls
made to this contract should have been required to have gone through the chain's governance process. This can vary from
chain to chain.

<details>

#### Function Signature

Expected function signature for the custom sudo handler:

```rust,ignore
{{#include ../../../packages/abstract-sdk/src/base/contract_base.rs:sudo}}
```

#### Message

There is no base message for this entry point. Your message will be the message that the endpoint accepts.

</details>

### Receive

The receive handler is a mutable entry point of the contract. It is similar to the `execute` handler but is specifically
geared towards handling messages that expect a `Receive` variant in the `ExecuteMsg`. Examples of this include but are
not limited to:

- Cw20 send messages
- Nois Network random number feed

<details>

#### Function Signature

Expected function signature for the custom receive handler:

```rust,ignore
{{#include ../../../packages/abstract-sdk/src/base/contract_base.rs:rec}}
```

#### Message

Called when the App's `ExecuteMsg::Receive` variant is called on the execute entry point.

```rust,ignore
{{#include ../../../packages/abstract-core/src/base.rs:exec}}
```

</details>

### Ibc Callback

The ibc callback handler is a mutable entry point of the contract. It is similar to the `execute` handler but is
specifically geared towards handling callbacks from IBC actions. Since interacting with IBC is an asynchronous process
we aim to provide you with the means to easily work with IBC. Our SDK helps you send IBC messages while this handler
helps you execute logic whenever the IBC action succeeds or fails. Our framework does this by optionally allowing you to
add callback information to any IBC action. A callback requires a unique `CallbackId` which is a `String`. The callback
handler takes an array of `(CallbackId, IbcCallbackFn)` tuples and matches any incoming callback on the
correct `CallbackId` for you. Every call to this handler is verified by asserting that the caller is the framework's
IBC-Client contract.

<!-- > We cover Abstract's IBC logic later in this book (TODO: add link to that section.) -->

<details>

#### Function Signature

```rust,ignore
{{#include ../../../packages/abstract-sdk/src/base/contract_base.rs:ibc}}
```

#### Message

Called when the App's `ExecuteMsg::IbcCallback` variant is called on the execute entry point. The receiving type is not
customizable but contains the IBC action acknowledgment.

```rust,ignore
{{#include ../../../packages/abstract-core/src/base.rs:exec}}
```

</details>

## Dependencies

There is another method accessible on the module builder, which is the `with_dependencies` function. As it states it
allows you to specify any smart contract dependencies that your module might require. This is a key requirement for
building truly composable and secure applications. We'll cover dependencies further
the [dependencies section](./8_dependencies.md).
