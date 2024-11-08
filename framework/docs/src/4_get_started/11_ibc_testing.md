# IBC Application Testing

One of the hardest steps in building an interchain application is testing. Due to the complexity of IBC and Cosmos SDK chains there are some trade-offs to consider in your approach to testing.

## Testing Tools

This page aims to provide you with a high-level overview of the available testing tools and how to use them. That way you can make an informed decision on which tool is best for your application's needs.

### Mock IBC Testing

The easiest way to test your CosmWasm IBC application is by using [cw-orchestrator's mock IBC environment](https://orchestrator.abstract.money/interchain/integrations/mock.html). This testing environment allows you to connect multiple `Mock` instances over a virtual IBC connection. Relaying is simulated and you can test your application's IBC logic without needing to deploy it to a live chain.

Type: `MockBech32Interchain`

**Advantages:**

- Easy to set up and use.
- Very fast to execute.
- Easily configurable.

**Disadvantages:**

- Does not support custom Cosmos SDK modules.
- Not end-to-end.
- Can't make use of existing on-chain infrastructure.

### Starship

[Starship](https://docs.cosmology.zone/starship) is a kubernetes-based Cosmos SDK environment spawner. It allows you to spin up multiple Cosmos SDK blockchain networks and connect them together. It includes a relayer, faucet and block explorer. Allowing you to test your application in a more realistic environment.

Type: `Starship`

**Advantages:**

- Access to Cosmos SDK modules from your smart contract.
- End-to-end testing.
- Can be made available for front-end testing.

**Disadvantages:**

- Slow to execute.
- Requires more setup and knowledge to use.
- Can only run for limited time due to resource constraints.

### Testnet

Type: `DaemonInterchain`

The final option is to deploy your application to a testnet. Doing this will ensure your application is tested in a real-world environment and makes it possible to start sharing what you've built with others. However, this approach is the most time-consuming and requires the most setup.

**Advantages:**

- Real-world testing.
- Can be shared with others.
- Can be used for marketing purposes.

**Disadvantages:**

- Slow to execute.
- Requires running your own relayer (or partnering with a relayer service).
- Testnets are often unstable.

## Tooling Conclusion

Before you start testing your application, consider the trade-offs between the different testing environments. For most applications, using the mock IBC environment is the best choice. However, if you need to test your application with custom Cosmos SDK modules, you should consider using Starship. Finally, if you're ready to share your application with the world, deploying to a testnet is the way to go.

## Testing Abstract Apps

We've created helpers for you to test your Abstract App in any of the aforementioned environments. The rest of this page will show you how to use them.

We'll be using the `abstract-client` crated which we covered in our section on testing. This crate includes a `AbstractInterchainClient` type that can be used to deploy and load abstract's infrastructure to any of the testing environments.

## Local & Mock Environments

For local and mock environments we provide the `AbstractInterchainClient::deploy_on` function. This function can take a `MockBech32Interchain` or `Starship` argument and it will deploy the Abstract contracts to the environment and set up the necessary IBC connections.

```rust
let interchain = MockBech32InterchainEnv::new(
        vec![("juno-1", "juno"), ("osmosis-1", "osmo")],
    );
let abstract_interchain = AbstractInterchainClient::deploy_on(&interchain);
// single-chain client
let juno_abstract: AbstractClient = abstract_interchain.client("juno-1");
```

## Testnet/Mainnet Environments

Abstract maintains deployments for a few testnets and for most mainnets. Instead of re-deploying Abstract to these networks you can make use of the `load_from` function on `AbstractInterchainClient`. To do this, first construct an `InterchainDaemon` object with the chains you'd like to use.

```rust
let interchain = DaemonInterchain::new(vec![
        (LOCAL_JUNO, None),
        (LOCAL_OSMO, None)
    ], &ChannelCreationValidator)?;

let abstract_interchain = AbstractInterchainClient::load_from(&interchain);
// single-chain client
let juno_abstract: AbstractClient = abstract_interchain.client("juno-1");
```
