# Testing Your Module

Testing your smart contracts is a crucial step in its development. Without proper testing you risk compromising the
accounts of your users and with it the funds that they hold. For that reason we expect modules to be thoroughly tested
before they are allowed on our platform.

This section of the documentation outlines the different testing methods. Each method is accompanied by an Abstract
helper. These helpers assist you in setting up your testing environment.

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

## Integration Testing

Integration testing your contract with Abstract involves deploying your contract and any of its dependencies to a mock environment where Abstract is deployed. To make this as easy as possible we've created a `abstract-client` package that you can use to deploy Abstract and any of your modules to a mock environment.

But first we need to cover some basics.

### Cw-orchestrator `Mock` environment

Most of our Abstract tests use cw-orchestrator's `Mock` struct that is backed by a `cw-multi-test::App` which you might be familiar with.

The `Mock` struct provides a simulation of the CosmWasm environment, enabling testing of contract functionalities.

```admonish info
`cw-orchestrator` is a CosmWasm scripting tool that we developed to improve the speed at which we can test and deploy our applications. We recommend reading the [cw-orchestrator documentation](../1_products/1_cw_orchestrator.md) if you are not yet familiar with it.
```

### Abstract Client

As previously mentioned you can use our `abstract-client` package to interact with a mocked instance of `Abstract`. Let's see how!

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

##### Your App Interface

The `Application<_, MockAppI<_>>` object returned from the `install_app` function is a wrapper around an `Account` that has an App installed on it (in this case `MockAppI`).

The `MockAppI` is a <a href="https://orchestrator.abstract.money/contracts/interfaces.html" target="_blank">cw-orchestrator *interface*</a> that exposes the contract's functions as methods. This allows you to easily interact with your module directly or as a different address.

```rust
{{ #include ../../../packages/abstract-client/tests/integration.rs:app_interface }}
```

## Local Daemon Testing

Once you have confirmed that your module works as expected you can spin up a local node and deploy Abstract + your app
onto the chain. You can do this by running the [test-local]() example, which uses a locally running juno daemon to
deploy to. At this point you can also test your front-end with the contracts.

```admonish info
Locally testing your Abstract deployment is difficult if it depends on other protocols, and those protocols don't make use of cw-orchestrator.
```

### Testing

You can test the module using the different provided methods.

1. **Integration testing:**
2. **Local Daemon:**


<!-- 
**Example**

The `Mock` encapsulates:

- A default sender for transactions.
- A state to map contract_id to its details.
- An emulation of the CosmWasm backend with app.

In this example, we use a setup function to initialize our test environment. The setup function is utilized to:

- Initialize the contract you want to test within the mock environment, the counter contract in this case.
- Upload and instantiate the contract.
- Retrieve essential details like code_id and contract address for further interactions.

This provides you with a streamlined approach to test and validate smart contract operations in a controlled setting.

```rust
/// Instantiate the contract in any CosmWasm environment
fn setup<Chain: CwEnv>(chain: Chain) -> CounterContract<Chain> {
    // Construct the counter interface
    let contract = CounterContract::new(CONTRACT_NAME, chain.clone());
    let admin = Addr::unchecked(ADMIN);

    // Upload the contract
    let upload_resp = contract.upload().unwrap();

    // Get the code-id from the response.
    let code_id = upload_resp.uploaded_code_id().unwrap();
    // or get it from the interface.
    assert_eq!(code_id, contract.code_id().unwrap());

    // Instantiate the contract
    let msg = InstantiateMsg { count: 1i32 };
    let init_resp = contract.instantiate(&msg, Some(&admin), None).unwrap();

    // Get the address from the response
    let contract_addr = init_resp.instantiated_contract_address().unwrap();
    // or get it from the interface.
    assert_eq!(contract_addr, contract.address().unwrap());

    // Return the interface
    contract
}

#[test]
fn count() {
    // Create a sender
    let sender = Addr::unchecked(ADMIN);
    // Create a user
    let user = Addr::unchecked(USER);
    // Create the mock
    let mock = Mock::new(&sender);

    // Set up the contract
    let contract = setup(mock.clone());

    // Increment the count of the contract
    contract
        // Set the caller to user
        .call_as(&user)
        // Call the increment function (auto-generated function provided by CounterExecuteMsgFns)
        .increment()
        .unwrap();

    // Get the count.
    use counter_contract::CounterQueryMsgFns;
    let count1 = contract.get_count().unwrap();

    // or query it manually
    let count2: GetCountResponse = contract.query(&QueryMsg::GetCount {}).unwrap();

    assert_eq!(count1, count2);

    // Check the count
    assert_eq!(count1.count, 2);
    // Reset
    use counter_contract::CounterExecuteMsgFns;
    contract.reset(0).unwrap();

    let count = contract.get_count().unwrap();
    assert_eq!(count.count, 0);

    // Check negative case
    let exec_res = contract.call_as(&user).reset(0);

    let expected_err = ContractError::Unauthorized {};
    assert_eq!(
        exec_res.unwrap_err().downcast::<ContractError>().unwrap(),
        expected_err
    );
}
``` -->