# CW-Orchestrator

!todo

Using cw-orchestrator for your smart-contract interactions reduces your testing/deployment overhead and improves both the code's readability and maintainability.

[cw-orchestrator](https://github.com/AbstractSDK/cw-orchestrator) is a smart-contract scripting library that simplifies smart-contract interactions. It allows you to re-use code between testing and deployments and acts as our primary tool in making Abstract's infrastructure highly available.

Here's a snippet that sets up the **complete Abstract SDK framework** on a cw-multi-test environment, and deploys the previously shown App contract to the framework.

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