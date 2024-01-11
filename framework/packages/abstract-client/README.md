# [Abstract Client](https://crates.io/abstract-client)

[![](https://img.shields.io/crates/v/abstract-client.svg)](https://crates.io/crates/abstract-client) [![Docs](https://docs.rs/abstract-client/badge.svg)](https://docs.rs/abstract-client)

The Abstract Client crate is a client-oriented way of interacting with the Abstract framework. The crate facilitates the following:

- Create and interact with Accounts.
- Install and interact modules.
- Publish modules.
- Configure and write integration tests.

## Getting Started

To get started you will need to create an `Abstract Client`. The client is the main entry point for interacting with the Abstract framework.

There are two ways to create a client, depending on the environment.

### Creating an `AbstractClient` For a Public Deployment

If you want to interact with a public deployment of Abstract (like a testnet or mainnet) then you can use the `Abstract::new` function like below:

```rust ignore
use abstract_client::AbstractClient;
use cw_orch::prelude::{Daemon, DaemonBuilder, networks};

// Specify a network to connect to (e.g. juno testnet, uni-6) 
let juno_testnet: Daemon = DaemonBuilder::default()
            .handle(rt.handle())
            .chain(networks::UNI_6)
            .build()?;

// Connect to the deployment
let client: AbstractClient<Daemon> = AbstractClient::new(juno_testnet)?;
```

### Creating an `AbstractClient` For a Local Deployment

When working with a local deployment (mock, or local daemon), you will need to deploy Abstract before you can interact with it. To do this you can use the `AbstractClient::builder` function which will deploy the infrastructure and return a client.

```rust
use cw_orch::prelude::*;
use abstract_client::AbstractClient;

let chain = Mock::new(&Addr::unchecked("sender"));

// Build the client, which will deploy the infrastructure
let client: AbstractClient<Mock> = AbstractClient::builder(chain).build()?;

Ok::<(), abstract_client::AbstractClientError>(())
```

## Interacting with the Client

The main function of the client is to construct `Account`, `Application` and `Publisher` objects.

### Creating an `Account`

To create an account you can use the `AbstractClient::account_builder` function. This function will return an [`AccountBuilder`](TODO) that you can configure to build an `Account`. If a claimed namespace is provided to the builder, the builder will return the `Account` that owns that namespace. In the other scenarios the builder will create a new account.

> Our examples will use the `Mock` environment for simplicity. However, the same functions can be used for any [`CwEnv`](https://docs.rs/cw-orch/latest/cw_orch/environment/trait.CwEnv.html).

```rust no_run
use cw_orch::prelude::*;
use abstract_client::{AbstractClient, Account, Application};
use abstract_app::mock::{mock_app_dependency::interface::MockAppDependencyInterface, MockInitMsg};

let chain = Mock::new(&Addr::unchecked("sender"));

// Construct the client
let client: AbstractClient<Mock> = AbstractClient::builder(chain).build()?;

// Build a new account.
let account: Account<Mock> = client.account_builder()
.build()?;

// Install an application.
let my_app: Application<Mock, MockAppDependencyInterface<Mock>> =
        account.install_app::<MockAppDependencyInterface<Mock>>(&MockInitMsg {}, &[])?;

Ok::<(), abstract_client::AbstractClientError>(())
```

See the [`AccountBuilder`](TODO) documentation for more information on how to customize an Account.

### Creating a `Publisher`

Creating a `Publisher` follows a similar process to creating an account. You can use the `AbstractClient::publisher_builder` function to create a `PublisherBuilder` that you can configure to build a `Publisher`.

However, unlike an `Account` a `Publisher` **must** have a namespace. If a namespace is not yet claimed, the builder will create a new account and claim the namespace. `Publisher` is simply a wrapper around an `Account`.

```rust
use cw_orch::prelude::*;
use abstract_client::{AbstractClient, Namespace, Publisher};

let chain = Mock::new(&Addr::unchecked("sender"));

// Construct the client
let client: AbstractClient<Mock> = AbstractClient::builder(chain).build()?;

// Build a Publisher
let publisher: Publisher<Mock> = client.publisher_builder(Namespace::new("my-namespace")?)
.build()?;

Ok::<(), abstract_client::AbstractClientError>(())
```

### Client Test Helpers

Additionally the client supports a set of helper functions you can explore <a href="TODO" target="_blank">here</a>.
