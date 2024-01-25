# Abstract Client

As previously mentioned you can use our `abstract-client` package to interact with any instance of `Abstract`. For this example we'll use the `Mock` environment for simplicity. However, the same functions can be used for any [`CwEnv`](https://docs.rs/cw-orch/latest/cw_orch/environment/trait.CwEnv.html).

> You can read the [`abstract-client` documentation](https://docs.rs/abstract-client/latest/abstract_client/) for more information.

#### Example

```rust
{{ #include ../../../packages/abstract-client/tests/integration.rs:build_client }}
```

These three lines:

- Created a mock environment to deploy to.
- Deployed Abstract to that environment and returned a client.

You can then start using the client to do all sorts of things. For example, you can set and query balances easily.

```rust
{{ #include ../../../packages/abstract-client/tests/integration.rs:balances }}
```

Then, you can use the client to create a `Publisher` to publish an App to the platform.

```rust
{{ #include ../../../packages/abstract-client/tests/integration.rs:publisher }}
```

Now that the App is published anyone can create an `Account` and install it!

```rust
{{ #include ../../../packages/abstract-client/tests/integration.rs:account }}
```

Et voila! You've just deployed Abstract and an App to a mock environment. You can now start testing your module.

The `Account` object also has some useful helper methods:

```rust
{{ #include ../../../packages/abstract-client/tests/integration.rs:account_helpers }}
```

You can explore more of its functions in [the type's documentation](https://docs.rs/abstract-client/latest/abstract_client/struct.Account.html).

##### Your App Interface

The `Application<_, MockAppI<_>>` object returned from the `install_app` function is a wrapper around an `Account` that has an App installed on it (in this case `MockAppI`).

The `MockAppI` is a <a href="https://orchestrator.abstract.money/contracts/interfaces.html" target="_blank">cw-orchestrator *interface*</a> that exposes the contract's functions as methods. This allows you to easily interact with your module directly or as a different address.

```rust
{{ #include ../../../packages/abstract-client/tests/integration.rs:app_interface }}
```
