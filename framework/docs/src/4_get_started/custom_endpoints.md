# Custom Module Endpoints

Default modules have a fixed set of endpoints that are defined by the base message type shown below.

```rust
{{#include ../../../packages/abstract-std/src/base.rs:exec}}
```

However, you might need to specify an endpoint on the top-level of your module, requiring changes to the base message type. As we'll discuss, this can be done by defining a custom module endpoint.

## Defining a Custom Module Endpoint

To define a custom module endpoint, you need to create a new message type that extends the base message type. This new message type should contain the messages supported by the base massages. In other words, it should be a superset of the base message type.

For example, if you require your contract to accept a `Receive(Cw20ReceiveMsg)` endpoint to handle `Cw20` deposits, then you can define a new message type as shown below.

```rust
pub enum CustomExecuteMsg {
    // Base message for your module type
    Base(abstract_app::std::app::BaseExecuteMsg),
    // Execute message for your module
    Module(AppExecuteMsg),
    /// Custom msg type
    Receive(cw20::Cw20ReceiveMsg),
}
```

Now the `App` object still expects the regular `ExecuteMsg` type, so you need to implement the `CustomExecuteHandler` trait for your custom message type to attempt to parse it into the base type.

When this `try_into_base` function returns an error, the `custom_execute` function will be called, allowing you to handle the custom message type.

```rust
impl CustomExecuteHandler<MyApp> for CustomExecuteMsg {
    type ExecuteMsg = crate::msg::ExecuteMsg;

    fn try_into_base(self) -> Result<Self::ExecuteMsg, Self> {
        match self {
            CustomExecuteMsg::Base(msg) => Ok(crate::msg::ExecuteMsg::from(msg)),
            CustomExecuteMsg::Module(msg) => Ok(crate::msg::ExecuteMsg::from(msg)),
            _ => Err(self),
        }
    }

    fn custom_execute(
        self,
        deps: DepsMut,
        env: Env,
        info: MessageInfo,
        module: MyApp,
    ) -> Result<Response, AppError> {
        match self {
            CustomExecuteMsg::Receive(cw20_msg) => {
                // Function that handles the custom message
                super::receive_handler(deps, env, info, module, cw20_msg)
            }
            _ => unreachable!(),
        }
    }
}
```

> We realize this introduces a lot of boilerplate code. Alternatively the `serde(untagged)` attribute could have been used but this increases wasm size and hurts performance significantly.

As a final change we need to update the `entrypoint` macro to include the custom message type.

```rust
#[cfg(feature = "export")]
abstract_app::export_endpoints!(APP, MyApp, CustomExecuteMsg);

abstract_app::cw_orch_interface!(
    APP,
    MyApp,
    MyAppInterface,
    CustomExecuteMsg
);
```

You can find a full example of a custom module endpoint in the [payment module](https://github.com/AbstractSDK/abstract/tree/main/modules/contracts/apps/payment) codebase.

### Cw-orch Function Support

To enable your endpoints to be called on the contract's interface you need to implement the `From` trait for the `CustomExecuteMsg` type. Where `AppExecuteMsg` is the module's inner execute message as shown above.

```rust
// Enable cw_orch api
impl From<AppExecuteMsg> for CustomExecuteMsg {
    fn from(value: AppExecuteMsg) -> Self {
        Self::Module(value)
    }
}
```
