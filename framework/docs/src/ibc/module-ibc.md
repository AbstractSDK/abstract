# Interchain Module Communication

## Introduction

[Interchain Abstract Accounts](./account-ibc.md) allows for permission-less actions to be executed by accounts across Abstract IBC deployments. However, this mechanism doesn't allow modules to talk to each other in a permissioned way. To clarify this sentence, imagine an Abstract module on a blockchain. This module wants to send a message to another module on a remote chain. In order to do so, they have 2 possibilities:

1. Send a message across IBC via the calling account. Using this method, it is impossible to verify that the content of the message was indeed sent by the origin module. This is good for permission-less entrypoints (for instance entry-points with funds deposit). This is not a good solution for permissioned entry-points, where the receiving module needs to make sure that the message was sent by the origin contract.
2. Create an account associated with the module and use this account to send messages. This solution works but necessitates a lot of access control and handling addresses and account ids across an app's deployment. This makes inter-operability very difficult

## Solving interchain module communication

In order to allow Modules to send messages to other modules across IBC, Abstract introduces Interchain Module Communication. This mechanism allows developers to send messages from a module directly to another module on a remote chain. This remote module can be any module and it allows interoperable permissionned actions between all Abstract modules. Let's see how to create a contract with Interchain Module capabilities.

### Sending a message

In order to send a message, a module needs to send a message on the current `ibc-client` module. To query the address of the `ibc-client` module, you can use the `app` or `adapter` variable available inside the endpoint like so:

```rust
let ibc_client_addr = app.modules(deps.as_ref()).module_address(IBC_CLIENT)?;
```

The [message](https://docs.rs/abstract-std/latest/abstract_std/ibc_client/enum.ExecuteMsg.html) looks like:

```rust
pub enum IbcClientExecuteMsg{
{{#include ../../../packages/abstract-std/src/native/ibc_client.rs:module-ibc-action}}
    ...,
}
```

- `host_chain` is the chain name of the destination chain
- `target_module` describes the module on which the message will be executed on the remote chain
- `msg` is the message that will be executed on the remote module. We explain in the section about [receiving a message](#receiving-a-message) how this message is used by the target module.
- `callback_info` is used to ask for a callback once the packet has been received and acknowledged. We explain more about this behavior in the [acks and callbacks section](#acknowledgements-and-callbacks)

### Receiving a message

In order for a module to receive an message coming from a remote Module, they need to implement the [`module-ibc`](../4_get_started/3_module_builder.md#module-ibc) endpoint. Here is what this endpoint looks like:

```rust
pub fn module_ibc(deps: DepsMut, env: Env, module: Module, msg: ModuleIbcMsg) -> Result<Response, Error>;
```

The `deps`, `env` and `module` variables are common to the `execute` endpoint and should be clear to you by now. If not here are some links to more documentation:

- `deps` and `env` are described in the <a target="blank" href="https://docs.cosmwasm.com/docs/smart-contracts/contract-semantics">CosmWasm documentation</a>
- `module` (or `app` or `adapter` usually) are described in the [Abstract SDK](../4_get_started/4_sdk.md) section of our docs

The `msg` variable describes the msg sent by the remote module <a target="blank" href="https://docs.rs/abstract-std/0.22.1/abstract_std/ibc/struct.ModuleIbcMsg.html">Link to the technical docs</a>:

```rust
{{#include ../../../packages/abstract-std/src/native/ibc.rs:module_ibc_msg}}
```

- `client_chain` is the name of the chain from which the call originates
- `source_module` describes the caller modules on the remote chains
- `msg` is the exact `Binary` data that was sent by the calling module on the remote chain. How this data is used is up to the receiving module. As per cosmwasm conventions, we advise using `from_json` and `to_json_binary` for serialization and deserialization of this field into the respective types.

The most important thing to never forget here is access control. Similarly to the `MessageInfo` struct usually used in smart contracts or in other execution endpoints, the `source_module` variable can be used to permission some entry-points inside your module ibc interactions. A good practice is to verify the namespace or the module-id directly present inside the `source_module` variable. For instance, the following code, will return an error if the source module doesn't have the same namespace as the receiving module. That way, you make sure that no other module than what was published within your own namespace is able to send module ibc messages to your app:

```rust
cosmwasm_std::ensure_eq!(
    source_module.namespace,
    APP_NAMESPACE,
    ContractError::Unauthorized {}
);
```

### Acknowledgements and Callbacks

#### Callback Request

The `callback_info` field allows conditional message execution after the successful IBC packet process. When this field is used, the result of the IBC message execution will be forwarded to the [`ibc_callback`] handler of the calling contract. Here is the structure of the callback request:

```rust
{{#include ../../../packages/abstract-std/src/native/ibc.rs:callback-info}}
```

- `id` is used to identify the callback id that will be called after the ibc execution. This works similarly to the reply id inside Abstract.
- `msg` is additional information that will be passed directly to the callback endpoint with the execution result. This allows developers to give context to the endpoint with the execution result.

#### Callback Execution

If a callback was asked for when sending a module IBC message, this callback will be called wether the execution was successful or not. A callback message will be sent on the ̀[`ibc_callback`](./4_get_started/3_module_builder.md#ibc-callback) endpoint. This endpoint has the following structure:

```rust
pub fn ibc_callback(deps: DepsMut, env: Env, module: Module, response_msg: IbcResponseMsg) -> Result<Response, Error>;
```

The `response_msg` variable describes the msg sent by the remote module <a target="blank" href="https://docs.rs/abstract-std/0.22.1/abstract_std/ibc/struct.IbcResponseMsg.html">(Link to the technical docs)</a>:

```rust
{{#include ../../../packages/abstract-std/src/native/ibc.rs:response-msg}}
```

As described above, the following fields are copied from the CallbackRequest:

- `id` is used to identify the callback id that is called after the ibc execution. This works similarly to the reply-id inside Abstract.
- `msg` is additional information that was passed directly by the callback request. This allows developers to give context to the endpoint with the execution result.

The last field `result` contains information about the execution result of the ibc interaction. Matching on the structure, catching errors, reverting state changes asending other messages, any execution is possible in this endpoint. This last part completes the interchain module communication flow and allows for full inter-operability!

## Specification of Interchain Module Communication

This part is not yet done. This is an outstanding TODO for the Abstract Team. If you're a module developer, you should have all the information you need in the previous sections.
