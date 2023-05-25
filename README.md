# Abstract App Module Template

The Abstract App Module Template is a starting point for developing composable smart-contracts, or "apps", that enable features or transform Abstract Accounts into standalone products. An app is instantiated for each Account individually and is migratable. Apps are allowed to perform actions on the Account and may also want to integrate with other Apps and Adapters installed on the Account. To learn more about Abstract Accounts, please see the [Abstract Accounts Documentation](TODO: Add link). To read more about apps, please see the [Abstract App Module Documentation](TODO: Add link).

## Getting Started

### Requirements

Learn more about the requirements for developing Abstract apps in the [Abstract App Module Documentation](TODO: Add link).

### Setup

To get started, clone this repository and run the following command:

```shell
chmod +x ./template-setup.sh
./template-setup.sh
```

The setup script will ask you for a name that will replace the "template" names in the repo. It also adds our Github CI repo to the repo.

## Using the Justfile

This repository comes with a `justfile`, which is a handy task runner that helps with building, testing, and deploying your Abstract app module.

### Installing Tools

To fully make use of the `justfile`, you need to install a few tools first. You can do this by simply running `just install-tools`. See [TODO: Add link] for more information.

### Available Tasks

Here are some of the tasks available in the `justfile`:

- `install-tools`: Install all the tools needed to run the tasks.
- `build`: Build everything with all features.
- `test`: Run all tests.
- `watch-test`: Watch the codebase and run tests on changes.
- `fmt`: Format the codebase (including .toml).
- `lint`: Lint-check the codebase.
- `lintfix`: Fix linting errors automatically.
- `watch`: Watch the codebase and run `cargo check` on changes.
- `check`: Check the codebase for issues.
- `deploy`: Deploy the App to a network.
- `wasm`: Optimize the contract.

You can see the full list of tasks available by running `just --list`.

### Deploying

Before attempting to deploy your app you need to add your mnemonic to the `.env` file. **Don't use a mnemonic that has mainnet funds for this.**

You can now use `just deploy` to run the `examples/deploy.rs` script. The script will deploy the app to the juno testnet. You can change the network by changing the `network` variable in the script.

## Contributing

We welcome contributions to the Abstract App Module Template! To contribute, fork this repository and submit a pull request with your changes. If you have any questions or issues, please open an issue in the repository and we will be happy to assist you.
