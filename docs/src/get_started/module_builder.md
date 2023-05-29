# Module Builder

Abstract provides multiple module bases, as detailed in our section on [modules](TODO: Add link). These base implementation provide you with the minimal state and configuration required to start building your modular application. After setting up your module base from our template you'll probably want to customize it. Our module builder pattern allows you to do just that. It also gives you a great overview on all the entry points to you module, and those that others have built.

## Overview

The builder pattern employed in building an Abstract module is a slight variation of the actual design pattern. Instead, the module builder lets you set custom entry point handlers at compile time, meaning you end up with a `const` value that is heavily optimized by the compiler. This ensures that the overhead of using Abstract has a negatable effect on both runtime and WASM binary size.

### App Type

To compose your module, first alias the app type by inserting your custom messages. These messages are inserted in the top-level entry point message types which we will cover later on. Here's an example:

```rust,ignore
{{#include ../../../packages/abstract-app/examples/counter.rs:counter_app}}
```

All of these messages can be customized and will be used to type-check the rest if your implementation.

## Build The App

Now that you have your type defined you can start using the builder. To do this, first construct the base version of the app:

```rust,ignore
{{#include ../../../packages/abstract-app/examples/counter.rs:new}}
```

The constructor takes three variables:

1. `contract_id`: The contract ID is a string that will be saved to a `cw2` storage item. This ensures that the contract can not be migrated to a different contract with a different function and also acts as an informational tag for off-chain processes.
2. `contract_version`: The contract version should be the version of the crate, it is also stored in the `cw2` item and is checked when performing migrations and on-chain dependency resolution.
3. `metadata`: An optional URL that can be used to retrieve data off-chain. Can be used with the Abstract Metadata Standard to automatically generate interactive front-end components for the module.

All these fields are used in a custom `ModuleData` store as well, along with the module's dependencies, which we will come back to later. Here's the definition of the `ModuleData` field:

```rust,ignore
{{#include ../../../packages/abstract-core/src/objects/module_version.rs:metadata}}
```

### Handlers

Then add whatever handler functions you need. These functions will be called whenever the specific endpoint is called on the module. A special feature about the functions is that we also insert the instance of your app into the function's attributes. This enables you to access the module struct in your code. You will learn why this is such a powerful feature in our section on the [Abstract SDK](sdk.md).

Here's an example of a module with some handlers set:

```rust,ignore
{{#include ../../../packages/abstract-app/examples/counter.rs:handlers}}
```

Let's go through all the available customizable handlers, the function signatures they require and how/when they get called.

### Instantiate

The execute entry point is the mutable entry point of the contract. Logic in this function can update the contract's state an trigger state changes in other contracts by calling them. It is where the majority of your contract's logic will reside.

#### Function Signature

```rust,ignore
{{#include ../../../packages/abstract-sdk/src/base/contract_base.rs:init}}
```

#### Message

Called when the App's instantiate entry point is called. Uses the struct's `module` field to customize the instantiation. Only this field is passed to the handler function.

```rust,ignore
{{#include ../../../packages/abstract-core/src/base.rs:init}}
```

### Execute

The execute entry point is a mutable entry point of the contract. Logic in this function can update the contract's state an trigger state changes in other contracts by calling them. It is where the majority of your contract's logic will reside.

#### Function Signature

```rust,ignore
{{#include ../../../packages/abstract-sdk/src/base/contract_base.rs:exec}}
```

#### Message

Called when the App's `ExecuteMsg::Module` variant is called on the execute entry point.

```rust,ignore
{{#include ../../../packages/abstract-core/src/base.rs:exec}}
```

### Query

The query entry point is the non-mutable entry point of the contract. Like its name implies it it used to retrieve data from the contract's state. This state retrieval can have a computation component but it can not alter the contract's or any other state.

#### Function Signature

```rust,ignore
{{#include ../../../packages/abstract-sdk/src/base/contract_base.rs:query}}
```

#### Message

Called when the App's `QueryMsg::Module` variant is called on the query entry point.

```rust,ignore
{{#include ../../../packages/abstract-core/src/base.rs:query}}
```

### Migrate

The migrate entry point is a mutable entry point that is called **after** a code_id change is applied to the contract. A migration in CosmWasm essentially swaps out the code that's executed at the contract's address while keeping the state as-is. The implementation of this function is often used to change the format of the contract's state by loading the data as the original format and overwriting it with a new format. All adapter base implementations already perform version assertions that make it impossible to migrate to a contract with a different ID or with a version that is lesser or equal to the old version.

#### Function Signature

```rust,ignore
{{#include ../../../packages/abstract-sdk/src/base/contract_base.rs:mig}}
```

#### Message

Called when the App's migrate entry point is called. Uses the struct's `module` field to customize the migration. Only this field is passed to the handler function.

```rust,ignore
{{#include ../../../packages/abstract-core/src/base.rs:migrate}}
```

### Reply

The reply entry point is a mutable entry point that is optionally called **after** a previous mutable action. It is often used by factory contracts to retrieve the contract of a newly instantiated contract. It essentially provides the ability perform callbacks on actions. A reply can be requested using CosmWasm's `SubMsg` type and requires a unique `ReplyId` which is a `u64`. The customizable handler takes an array of `(ReplyId, ReplyFn)` tuples and matches any incoming reply on the correct `ReplyId` for you.

#### Function Signature

```rust,ignore
{{#include ../../../packages/abstract-sdk/src/base/contract_base.rs:reply}}
```

#### Message

There is no customizable message associated with this entry point.

### Sudo

The sudo entry point is a mutable entry point that can only be called by the chain's governance module. I.e. any calls made to this contract should have been required to have gone through the chain's governance process. This can vary from chain to chain.

#### Function Signature

```rust,ignore
{{#include ../../../packages/abstract-sdk/src/base/contract_base.rs:sudo}}
```

#### Message

There is no base message for this entry point. Your message will be the message that the endpoint accepts.

### Receive

The receive handler is a mutable entry point of the contract. It is similar to the `execute` handler but is specifically geared towards handling messages that expect a `Receive` variant in the `ExecuteMsg`. Examples of this include but are not limited to:

- Cw20 send messages
- Nois Network random number feed

#### Function Signature

```rust,ignore
{{#include ../../../packages/abstract-sdk/src/base/contract_base.rs:rec}}
```

#### Message

Called when the App's `ExecuteMsg::Receive` variant is called on the execute entry point.

```rust,ignore
{{#include ../../../packages/abstract-core/src/base.rs:exec}}
```

### Ibc Callback

The ibc callback handler is a mutable entry point of the contract. It is similar to the `execute` handler but is specifically geared towards handling callbacks from IBC actions. Since interacting with IBC is an asynchronous process we aim to provide you with the means to easily work with IBC. Our SDK helps you send IBC messages while this handler helps you execute logic whenever the IBC action succeeds or fails. Our framework does this by optionally allowing you to add callback information to any IBC action. A callback requires a unique `CallbackId` which is a `String`. The callback handler takes an array of `(CallbackId, IbcCallbackFn)` tuples and matches any incoming callback on the correct `CallbackId` for you. Every call to this handler is verified by asserting that the caller is the framework's IBC-Client contract.

> We cover Abstract's IBC logic later in this book (TODO: add link to that section.)

#### Function Signature

```rust,ignore
{{#include ../../../packages/abstract-sdk/src/base/contract_base.rs:ibc}}
```

#### Message

Called when the App's `ExecuteMsg::IbcCallback` variant is called on the execute entry point. The receiving type is not customizable but contains the IBC action acknowledgment.

```rust,ignore
{{#include ../../../packages/abstract-core/src/base.rs:exec}}
```

<!-- - `with_execute`: 
- `with_instantiate`: Called when the App's `InstantiateMsg` is called on the instantiate entry point.
- `with_query`: Called when the App's `QueryMsg::Module` is called on the query entry point.
- `with_migrate`: Called when the App's `MigrateMsg` is called on the migrate entry point.
- `with_replies`: Called when the App's reply entry point is called. Matches the function's associated reply-id.
- `with_sudo`: Called when the App's `SudoMsg` is called on the sudo entry point.
- `with_receive`: Called when the App's `ExecuteMsg::Receive` variant is called on the execute entry point.
- `with_ibc_callbacks`: Called when the App's `ExecuteMsg::IbcCallback` is called on the execute entry point. Matches the callback's callback ID to its associated function. -->

## Dependencies

Theres is one additional contractor method available on the module builder and that's the `with_dependencies` function. As it states it allows you to specify any smart-contract dependencies that your application might require. This is a key requirement for building truly composable and secure applications. We'll cover dependencies further in [our section on them]()

## Summary

The Abstract SDK allows you to easily construct modules by using our low-overhead smart-contract builder. By employing this pattern you re-use the base contract's code, allowing you to focus on the ideas that make your product unique.

In the next section we'll cover how you can use the module object that we make available in the function handlers to write highly functional smart-contract code.

Ever wanted to swap on any cosmos DEX with only one line of code? Look no further!
