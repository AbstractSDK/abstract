# [Abstract Client](https://crates.io/abstract-client)

[![](https://img.shields.io/crates/v/abstract-client.svg)](https://crates.io/crates/abstract-client) [![Docs](https://docs.rs/abstract-client/badge.svg)](https://docs.rs/abstract-client)

This crate provides a central `AbstractClient` object that facilitates interacting with the Abstract framework. The client exposes two builders, an `account_builder` and `publisher_builder` that can be used to retrieve and create accounts.

Applications and their dependencies can then be installed on the `Account` using the `install_app` or `install_app_with_dependencies` functions.

## Creating an `AbstractClient`

To get started you will need to create an `Abstract Client`. The client is the main entry point for interacting with the Abstract framework.

There are two ways to create a client, depending on the environment.

### For a Public Deployment

If you want to interact with a public deployment of Abstract (like a testnet or mainnet) then you can use the `Abstract::new` function like below:

```rust no_run
use abstract_client::AbstractClient;
use cw_orch::prelude::{Daemon, DaemonBuilder, networks};

// Specify a network to connect to (e.g. juno testnet, uni-6) 
let juno_testnet: Daemon = Daemon::builder(networks::UNI_6).build()?;

// Connect to the deployment
let client: AbstractClient<Daemon> = AbstractClient::new(juno_testnet)?;
Ok::<(), anyhow::Error>(())
```

### For a Local Deployment

When working with a local integration testing (Mock), you will need to deploy Abstract before you can interact with it. To do this you can use the `AbstractClient::builder` function which will deploy the infrastructure when the builder is built and return a client.

**For Mock:** 
```rust
use cw_orch::prelude::*;
use abstract_client::AbstractClient;

let chain = MockBech32::new("mock");

// Build the client, which will deploy the infrastructure
let client: AbstractClient<MockBech32> = 
            AbstractClient::builder(chain)
            // ... Configure builder 
            .build_mock()?;

Ok::<(), abstract_client::AbstractClientError>(())
```

## Interacting with the Client

The main function of the client is to construct `Account`, `Application` and `Publisher` objects.

### Creating an `Account`

To create an account you can use the `AbstractClient::account_builder` function. This function will return an [`AccountBuilder`](https://docs.rs/abstract-client/latest/abstract_client/struct.AccountBuilder.html) that you can configure to build an `Account`. If a claimed namespace is provided to the builder, the builder will return the `Account` that owns that namespace. In the other scenarios the builder will create a new account.

> Our examples will use the `MockBech32` environment for simplicity. However, the same functions can be used for any [`CwEnv`](https://docs.rs/cw-orch/latest/cw_orch/environment/trait.CwEnv.html).

```rust no_run
use cw_orch::prelude::*;
use abstract_client::{AbstractClient, Account, Application};
use abstract_app::mock::{mock_app_dependency::interface::MockAppI as App, MockInitMsg as AppInitMsg};

let chain = MockBech32::new("mock");

// Construct the client
let client: AbstractClient<MockBech32> = AbstractClient::builder(chain).build_mock()?;

// Build a new account.
let account: Account<MockBech32> = client.account_builder().build()?;

// Install an application.
let my_app: Application<MockBech32, App<MockBech32>> =
        account.install_app::<App<MockBech32>>(&AppInitMsg {}, &[])?;

Ok::<(), abstract_client::AbstractClientError>(())
```

See the [`AccountBuilder`](https://docs.rs/abstract-client/latest/abstract_client/struct.AccountBuilder.html) documentation for more information on how to customize an Account.

### Creating a `Publisher`

Creating a `Publisher` follows a similar process to creating an account. You can use the `AbstractAccount::publisher` function to create a `Publisher` from an account. Don't forget the claim a namespace for your account ! 

However, unlike an `Account` a `Publisher` **must** have a namespace. If a namespace is not yet claimed, the builder will create a new account and claim the namespace. `Publisher` is simply a wrapper around an `Account`.

The `Publisher` can then be used to publish modules to the abstract app store, as shown below.

After the `App` is published it can be installed on accounts.

```rust
use cw_orch::prelude::*;
use abstract_client::{AbstractClient, Namespace, Publisher, Application};
use abstract_app::mock::{mock_app_dependency::interface::MockAppI, MockInitMsg};

let chain = MockBech32::new("mock");

// Construct the client
let client: AbstractClient<MockBech32> = AbstractClient::builder(chain).build_mock()?;

// Build a Publisher
let publisher: Publisher<MockBech32> = client.account_builder().namespace(Namespace::new("tester")?).build()?.publisher()?;

publisher.publish_app::<MockAppI<_>>()?;

// Install the published app
let app: Application<MockBech32, MockAppI<MockBech32>> =
        publisher.account().install_app::<MockAppI<MockBech32>>(&MockInitMsg {}, &[])?;


Ok::<(), abstract_client::AbstractClientError>(())
```

### Fetching an `Account`

If you've previously created an `Account` and wish to retrieve it, you can use the `AbstractClient::account_from` function. This function accepts three different types of input:

- `Namespace` - If the namespace is claimed, the function will return the `Account` that owns the namespace.
- `AccountId` - If this `AccountId` exists, the function will return the `Account` with that `AccountId`.
- App `Addr` - If there's an `App` installed on the account you can provide its `Addr` to retrieve the `Account` that it is installed on.

```rust
use cw_orch::prelude::*;
use abstract_client::{AbstractClient, Namespace, Account};
use abstract_app::mock::{mock_app_dependency::interface::MockAppI, MockInitMsg};

let chain = MockBech32::new("mock");

// Construct the client
let client: AbstractClient<MockBech32> = AbstractClient::builder(chain).build_mock()?;

let namespace = Namespace::new("some-namespace")?;

// Build a new account.
let account: Account<MockBech32> = client.account_builder().namespace(namespace.clone()).build()?;

// Fetch the account
let fetched_account: Account<MockBech32> = client.account_from(namespace)?;

Ok::<(), abstract_client::AbstractClientError>(())
```

### Client Test Helpers

Additionally the client supports a set of helper functions you can explore <a href="https://docs.rs/abstract-client/latest/abstract_client/struct.AbstractClient.html" target="_blank">here</a>.
