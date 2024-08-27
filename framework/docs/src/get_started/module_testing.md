# Testing Your Module

Testing your smart contracts is a crucial step in its development. Without proper testing you risk compromising the
accounts of your users and with it the funds that they hold. For that reason we expect modules to be thoroughly tested
before they are allowed on our platform.

This section of the documentation outlines the different testing methods. Each method is accompanied by an Abstract
helper. These helpers assist you in setting up your testing environment.

## Integration Testing

Integration testing your contract with Abstract involves deploying your contract and any of its dependencies to a mock environment where Abstract is deployed. To make this as easy as possible we've created a `abstract-client` package that you can use to deploy Abstract and any of your modules to a mock environment. We will cover this client in the next section.

But first we need to cover some basics.

### Cw-orchestrator `Mock` environment

Most of our Abstract tests use cw-orchestrator's `Mock` struct that is backed by a `cw-multi-test::App` which you might be familiar with.

The `Mock` struct provides a simulation of the CosmWasm environment, enabling testing of contract functionalities.

```admonish info
`cw-orchestrator` is a CosmWasm scripting tool that we developed to improve the speed at which we can test and deploy our applications. We recommend reading the [cw-orchestrator documentation](../products/cw_orchestrator.md) if you are not yet familiar with it.
```

**Example**

```rust,ignore
{{ #include ../../../packages/abstract-client/tests/integration.rs:mock_integration_test}}
```

**Details**

The `Mock` encapsulates:

- A default sender for transactions.
- A state to map contract_id to its details.
- An emulation of the CosmWasm backend with app.

In this example, we use a setup function to initialize our test environment. The setup function is utilized to:

- Initialize the contract you want to test within the mock environment, the counter contract in this case.
- Upload and instantiate the contract.
- Retrieve essential details like code_id and contract address for further interactions.

This provides you with a streamlined approach to test and validate smart contract operations in a controlled setting.

## Local Daemon Testing

Once you have confirmed that your module works as expected you can spin up a local node and deploy Abstract + your app onto the chain. You can do this by running the [local_daemon](https://github.com/AbstractSDK/app-template/blob/main/examples/local_daemon.rs) example, which uses a locally running juno daemon to
deploy to. At this point you can also test your front-end with the contracts.

```admonish info
Testing your application on a local daemon is difficult if it depends on other protocols, and those protocols don't make use of cw-orchestrator as there is no easy way to deploy them to the local daemon.
```

## Unit-testing

The lowest level of testing is *unit testing*. Unit tests allow you to easily test complex, self-contained logic. Because unit tests should be self-contained, any queries made to other contracts need to be mocked. These mocks act as "query catchers", allowing you to specify a response for a specific query.

Sadly constructing these mock queries is time-consuming and involves a lot of boilerplate. Additionally, there are queries that your module should always support as they are part of its base implementation. For those reasons we created an `abstract-testing` package.

The `abstract-testing` provides you with some small abstractions that allow you to mock Smart and Raw queries with ease.

```admonish info
What's the difference between a Smart and a Raw query?

- **Smart Queries:** A smart query is a query that contains a message in its request. It commonly involves computation on the queried contract. After this optional computation and state loading, the contract responds with a *ResponseMsg*. Mocking this type of query involves matching the serialized query request message (`Binary`) to a specific message type and returning a serialized response. Any expected computation needs to be mocked as well.

- **Raw Queries:** A raw query is a simple database key-value lookup. To mock this type of query you need to provide a mapping of the raw key to a raw value. The returned value then needs to be interpreted correctly according to the store's type definitions.
```

### Mock Querier

The `abstract-testing` package contains
a <a href="https://docs.rs/abstract-testing/latest/abstract_testing/struct.MockQuerierBuilder.html" target="_blank">`MockQuerierBuilder`</a>.
It uses the common **builder pattern** to allow for efficient mock construction. Let's see how!

#### Mocking Smart Queries

Mocking a smart-query with
the <a href="https://docs.rs/abstract-testing/latest/abstract_testing/struct.MockQuerierBuilder.html" target="_blank">`MockQuerierBuilder`</a>
is easy! You do it by calling the `with_smart_handler` function.

##### Example

```rust
{{ #include ../../../packages/abstract-testing/src/mock_querier.rs:smart_query}}
```

#### Mocking Raw Queries

Instead of manually mapping the key-value relation and it's types, we can use the available contract storage types.
Using the storage types ensures that the mock and its data operations are the same as in the actual implementation. It
also saves us a lot of work related to key serialization.

This approach allow you to easily map `Item` and `Map` datastores.

```admonish warning
Multi-index maps are currently not supported. PRs on this issue are welcome! 🤗
```

##### Example

```rust
{{ #include ../../../packages/abstract-testing/src/mock_querier.rs:raw_query }}
```

#### `Items` and `Maps`

The `MockQuerierBuilder` also provides a `with_items` and `with_maps` function. These functions allow you to easily mock `Item` and `Map` datastores.

#### Abstract Querier

The easiest and best way to start using the querier is to use
the `AbstractMockQuerierBuilder::mocked_account_querier_builder()` method. This method sets up a mock querier with an
initial Abstract Account.
