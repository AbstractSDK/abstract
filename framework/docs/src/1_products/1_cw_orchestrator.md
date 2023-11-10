# Testing And Deployment
## CW-Orchestrator

The <a href="https://github.com/AbstractSDK/cw-orchestrator" target="_blank">cw-orchestrator</a> package is a Rust-native scripting tool designed to simplify interactions with CosmWasm smart contracts. It provides a set of macros that generate type-safe interfaces for your contracts, enhancing readability and maintainability.


Cw-Orchestrator is integrated with the following environments;
- cw-multi-test (mock testing)
- test-tube (integration testing)
- live daemons (localnets / testnets / mainnets)
Furthermore, cw-orchestrator allows for code reusability between testing and deployments, establishing itself as our
primary tool in making Abstract's infrastructure highly available.

### Usage

Here's a snippet that sets up the **complete Abstract SDK framework** on a cw-multi-test environment, and deploys the
Counter App to the framework.

```rust,no_run
// Create a sender and instantiate the mock environment
let sender = Addr::unchecked("sender");
let mock = Mock::new(&sender);

// Construct the counter interface (a wrapper around the contract's entry points)
let contract = CounterApp::new(COUNTER_ID, mock.clone());

// Deploy Abstract to the mock
let abstr_deployment = Abstract::deploy_on(mock, Empty{})?;

// Create a new account to install the app onto
let account =
    abstr_deployment
        .account_factory
        .create_default_account(GovernanceDetails::Monarchy {
            monarch: sender.to_string(),
        })?;

// Claim the namespace so app can be deployed
abstr_deployment
    .version_control
    .claim_namespace(1, "my-namespace".to_string())?;

// Deploy the app!
contract.deploy(APP_VERSION.parse()?)?;
```

For more details on how to use cw-orchestrator, please refer to
the <a href="https://orchestrator.abstract.money/" target="_blank">cw-orchestrator Documentation</a>, where you can find
a quick start and a detailed guide on how to use the tool with your smart contracts, supported chains and more. Also,
check out the <a href="https://github.com/AbstractSDK/cw-orchestrator" target="_blank">cw-orchestrator Github Repo</a>
for more details about the tool's code.