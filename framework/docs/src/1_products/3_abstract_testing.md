
[//]: # (TODO: Re-introduce when this becomes relevant.)

# Abstract-Testing

The `abstract-testing` package is Abstract framework's testing utility, focusing on mocking and querying
functionalities. These utilities are essential for simulating various scenarios, ensuring the correctness of the
framework's functionalities, and facilitating robust unit testing.

## Features

- ️**Mock Data Creation** 🛠: Easily create mock data with predefined data for assets, contracts, accounts and more.
- **Abstract Naming Service Integration** 🌐: Add mock assets into the Abstract Naming Service (ANS) for testing
  purposes.
- **Flexible Configuration** 🔧: Adjust registry addresses, set up mock ANS hosts, and more.
- **Assertion Tools** ✅: Assert the existence of accounts, assets, map entries and more.
- **Predefined Test Scenarios** 📝: Run through common test scenarios with ease.
- **Build & Execute** 🔄: Construct mock queries and execute them to test various functionalities.

## Usage

Add the `abstract-unit-test-utils` dependency to your Cargo.toml file:

```toml
[dependencies]
abstract-unit-test-utils = "0.18.0"
```

For more information about the available types and methods, please refer to
the <a href="https://docs.rs/abstract-testing/0.18.0/abstract_testing/" target="_blank">Abstract-Testing
Documentation</a>.

You can find the latest version of the package on [crates.io](https://crates.io/crates/abstract-testing).

## Example

```rust,no_run
use abstract_unit_test_utils::MockQuerierBuilder;
use abstract_unit_test_utils::prelude::*;

#[test]
fn returns_account_owner() -> VersionControlTestResult {
    let mut deps = mock_dependencies();
    // Set up mock querier with the account
    deps.querier = AbstractMockQuerierBuilder::default()
        .account(TEST_ACCOUNT, 0)
        .build();
    mock_init_with_account(deps.as_mut(), true)?;

    let account_owner =
        query_account_owner(&deps.as_ref().querier, &Addr::unchecked(TEST_ACCOUNT), 0)?;

    assert_that!(account_owner).is_equal_to(Addr::unchecked(OWNER));
    Ok(())
}
```
