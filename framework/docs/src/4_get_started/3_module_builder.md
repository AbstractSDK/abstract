# Module Builder

Abstract provides multiple module bases, as detailed in our section on [modules](../3_framework/6_module_types.md).
These base implementation provide you with the minimal state and configuration required to start building your modular
application. After setting up your module base from our template you'll probably want to customize it. Our module
builder pattern allows you to do just that. It also gives you a great overview on all the entry points to your module,
and those that others have built.

## Overview

The builder pattern employed in building an Abstract module is a slight variation of the actual design pattern. Instead,
the module builder lets you set custom entry point handlers at compile time, meaning you end up with a `const` value
that is heavily optimized by the compiler. This ensures that the overhead of using Abstract has little effect on
both the code's runtime and WASM binary size.

```admonish info
The code-snippets in this example can be found in the <a href="https://github.com/AbstractSDK/abstract/blob/main/framework/packages/abstract-app/examples/counter.rs" target="_blank">counter app example</a>.
```

In this tutorial we will be working on an [`App` module](../3_framework/6_module_types.md#apps).

### App Type

To set up your App module, start by integrating your custom messages. These messages are inserted in the
top-level entry point message types, which will be discussed in more detail later. Here's an example:

```rust,ignore
{{#include ../../../packages/abstract-app/examples/counter.rs:counter_app}}
```

All of these messages can be customized and will be used to type-check the rest of your implementation.

## Build The App

Now that you have defined your type, you can begin using the builder. To initiate this, first create the base version of
the app:

```rust,ignore
{{#include ../../../packages/abstract-app/examples/counter.rs:new}}
```

The constructor takes three variables:

1. `module_id`: The module ID is a string that will be saved to a `cw2` storage item. It's an important security
   measure as this ensures that the contract can not be migrated to a different contract with a different function and
   also acts as an informational tag for off-chain processes.
2. `contract_version`: The contract version should be the version of the crate, it is also stored in the `cw2` item and
   is checked when performing migrations and on-chain dependency resolution.
3. `metadata`: An optional URL that can be used to retrieve data off-chain. Can be used with the Abstract Metadata
   Standard to automatically generate interactive front-end components for the module.

All these fields are used in a custom `ModuleData` store as well, along with the module's dependencies, which we will
come back to later. Here's the definition of the `ModuleData` field:

```rust,ignore
{{#include ../../../packages/abstract-std/src/objects/module_version.rs:metadata}}
```

All this data is stored on-chain when the App is instantiated.

### Handlers

The app can then be customized by adding whatever handler functions you need. These functions are executed whenever a
specific endpoint is called on the module. A special feature about the functions is that we insert the instance of your
module into the function's attributes. This enables you to access the module struct in your code. You will learn why
this is such a powerful feature in the next section on the [Abstract SDK](./4_sdk.md).

Here's an example of a module with some handlers set:

```rust,ignore
{{ #include ../../../packages/abstract-app/examples/counter.rs:handlers }}
```

These handlers are functions that allow you to customize the smart contract's behavior. For example, here's a
custom `execute` handler that updates the contract's config state.

```rust
{{ #include ../../../packages/abstract-app/examples/counter.rs:execute }}
```

```admonish info
You can find more application code to read in our <a href="https://github.com/AbstractSDK/awesome-abstract" target="_blank">💥 Awesome Abstract repository 💥</a>.
```

The available handlers are:

- `with_execute`: Called when the App's `ExecuteMsg` is called on the instantiate entry point.
- `with_instantiate`: Called when the App's `InstantiateMsg` is called on the instantiate entry point.
- `with_query`: Called when the App's `QueryMsg::Module` is called on the query entry point.
- `with_migrate`: Called when the App's `MigrateMsg` is called on the migrate entry point.
- `with_replies`: Called when the App's reply entry point is called. Matches the function's associated reply-id.
- `with_sudo`: Called when the App's `SudoMsg` is called on the sudo entry point.
- `with_receive`: Called when the App's `ExecuteMsg::Receive` variant is called on the execute entry point.
- `with_ibc_callbacks`: Called when the App's `ExecuteMsg::IbcCallback` is called on the execute entry point. Matches
  the callback's callback ID to its associated function.

In the case of adapters, the handlers are the same, except for `with_migrate` and `with_sudo` that are missing for
reasons we explain in the [adapter section](../3_framework/6_module_types.md#adapters).

For a full overview of the list of handlers available, please refer to the respective module type documentation:

- <a href="https://docs.rs/abstract-app/latest/abstract_app/state/struct.AppContract.html" target="_blank">App</a>
- <a href="https://docs.rs/abstract-adapter/latest/abstract_adapter/state/struct.AdapterContract.html" target="_blank">
  Adapter</a>

Below, we examine each handler in greater detail. The `base` fields and variants mentioned in the messages below are
defined
by [the base module type](../3_framework/6_module_types.md) that you chose to use, an `App` in this case.

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
{{#include ../../../packages/abstract-std/src/base.rs:init}}
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
{{#include ../../../packages/abstract-std/src/base.rs:exec}}
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
{{#include ../../../packages/abstract-std/src/base.rs:query}}
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
{{#include ../../../packages/abstract-std/src/base.rs:migrate}}
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
{{#include ../../../packages/abstract-std/src/base.rs:exec}}
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
{{#include ../../../packages/abstract-std/src/base.rs:exec}}
```

</details>

## Dependencies

There is another method accessible on the module builder, which is the `with_dependencies` function. As it states it
allows you to specify any smart contract dependencies that your module might require. This is a key requirement for
building truly composable and secure applications. We'll cover dependencies further
the [dependencies section](./8_dependencies.md).

## Summary

The Abstract SDK allows you to easily construct modules by using our low-overhead smart contract builder. By employing
this pattern you re-use the base contract's code, allowing you to focus on the ideas that make your product unique.

In the next section we'll cover how you can use the module object that we make available in the function handlers to
write highly functional smart contract code.

Ever wanted to swap on any cosmos DEX with only one line of code? Look no further!
