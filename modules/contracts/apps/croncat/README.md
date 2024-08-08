# CronCat App module

## Description

The CronCat app module is used to automate Abstract Account actions or other modules

## Features
- Communication with [croncat contracts](https://docs.cron.cat/docs/deployed-contracts/)
- Create task which contains:
  - Onchain actions
  - How frequent actions should be executed
  - It may contain: Boundary, "If this then that"... For more details on "tasks" refer to [croncat documentation](https://docs.cron.cat/docs/task-anatomy/)
- Remove task
- Refill task
- It uses Abstract's account balance for creation and refilling a croncat task 

## Installation

To use the Croncat App Module in your Rust project, add the following dependency to your `Cargo.toml`:
```toml
[dependencies]
croncat-app = { git = "https://github.com/AbstractSDK/abstract.git", tag="<latest-tag>", default-features = false }
```

## Usage with the Abstract SDK
To interact with a croncat, you first need to retrieve the croncat using the Croncat App. Here's a basic example in Rust:
```rust
// Retrieve the croncat
use croncat_app::CronCatInterface;
...

let cron_cat = app.croncat(deps.as_ref());
let task = CronCatTaskRequest {
  // Task request
};
let assets = AssetList::from(
  // Assets
).into();
let task_id = "custom_id";
let task_msg = cron_cat.create_task(task, task_id, assets)?;
```

## Why use the CronCat App?

### Scheduled on-chain actions
By using the CronCat App, developers and abstract users can create scheduled on-chain actions. This reduces the risk of human error and reliance on centralized intermediaries.

### Time limited actions
The CronCat App can help to execute certain actions within time limits (for example claims and sales).

### Use cases
- **[DCA](../dca/README.md)**: Periodically convert one asset to the other
- **Market**: Buying some Asset(for example NFT) that's available only in known limited time interval
- **Games**: Periodic `tick` is important event for some games 

### Documentation
- **CronCat Interface**: For a detailed look at the CronCat interface, refer to the [Rust trait interface](https://github.com/AbstractSDK/abstract/blob/42d967c3b13c1194e92a03244c8c5b2843726d12/modules/contracts/apps/croncat/src/api.rs#L21).
- **Apps Documentation**: Comprehensive information about apps can be found in the [official documentation](https://docs.abstract.money/3_framework/7_module_types.html#apps).


## Documentation

- **App Documentation**: Comprehensive information about adapters can be found in the [official documentation](https://docs.abstract.money/3_framework/6_module_types.html#apps).

## Contributing

If you have suggestions, improvements or want to contribute to the project, we welcome your input on [GitHub](https://github.com/AbstractSDK/abstract).

## Community
Check out the following places for support, discussions & feedback:

- Join our [Discord server](https://discord.com/invite/uch3Tq3aym)
