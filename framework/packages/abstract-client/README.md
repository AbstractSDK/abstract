# [Abstract Client](https://crates.io/abstract-client)

[![](https://img.shields.io/crates/v/abstract-client.svg)](https://crates.io/crates/abstract-client) [![Docs](https://docs.rs/abstract-client/badge.svg)](https://docs.rs/abstract-client)

The Abstract Client crate is a client-oriented way of interacting with the Abstract framework. The crate facilitates the following:

- Create and interact with Accounts.
- Install and interact modules.
- Publish modules.
- Configure and write integration tests.

## Getting Started

To get started you will need to create an Abstract Client. The client is the main entry point for interacting with the Abstract framework.

There are two ways to create a client, depending on the environment.

### Creating an `AbstractClient` For a Public Deployment

If you want to interact with a public deployment of Abstract (like a testnet or mainnet) then you can use the `Abstract::new` function like below:

```rust
use abstract_app::mock::interface::MockAppInterface;
use cw_orch::prelude::{Mock, DaemonBuilder, networks};
use abstract_client::client::AbstractClient;

// Specify a network to connect to (e.g. juno testnet, uni-6) 
let juno_testnet = DaemonBuilder::default()
            .handle(rt.handle())
            .chain(networks::UNI_6)
            .build()?;

// Connect to the deployment
let client: AbstractClient<Daemon> = AbstractClient::new(juno_testnet)?;
```

### Creating an `AbstractClient` For a Local Deployment

When working with a local deployment (mock, or local daemon), you will need to deploy Abstract before you can interact with it. To do this you can use the `AbstractClient::builder` function which will deploy the infrastructure and return a client.

Example:

```rust
use abstract_app::mock::interface::MockAppInterface;
use cw_orch::prelude::{Mock, DaemonBuilder};
use abstract_client::{client::AbstractClient, publisher::Publisher};

let client: AbstractClient<Mock> = AbstractClient::builder("sender").build()?;
let namespace = N"tester";
let publisher: Publisher<Mock> = client
    .publisher_builder(namespace)
    .build()?;
publisher.publish_app::<MockAppInterface<Mock>>()?;
```
