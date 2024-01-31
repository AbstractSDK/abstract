# Abstract App Module Template

The Abstract App Module Template is a starting point for developing composable smart-contracts, or "Apps" on the Abstract platform. An App is instantiated for each Account individually and is migratable. Apps are allowed to perform actions on Abstract Accounts and can integrate with other Apps and Adapters installed on the Account. To learn more about Abstract Accounts, please see the [abstract accounts documentation](https://docs.abstract.money/4_framework/4_architecture.html). To read more about apps, please see the [app module documentation](https://docs.abstract.money/4_framework/7_module_types.html).

## Getting Started

### Requirements

Learn more about the requirements for developing Abstract apps in the [getting started documentation](https://docs.abstract.money/4_get_started/1_index.html).

### Setup

To get started, clone this repository and run the following command:

```shell
chmod +x ./template-setup.sh
./template-setup.sh
```

The setup will suggest you to install a few tools that are used in the template. You can skip this step if you already have them installed or if you're not planning on using them.

## Using the Justfile

This repository comes with a [`justfile`](https://github.com/casey/just), which is a handy task runner that helps with building, testing, and publishing your Abstract app module.

### Installing Tools

To fully make use of the `justfile`, you need to install a few tools first. You can do this by simply running `just install-tools`. See [tools used the template](https://docs.abstract.money/3_get_started/2_installation.html?#tools-used-in-the-template) for more information.

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
- `publish {{chain-id}}`: Publish the App to a network.
- `wasm`: Optimize the contract.
- `schema`: Generate the json schemas for the contract
- `ts-codegen`: Generate the typescript client code for the contract
- `ts-publish`: Publish the typescript client code to npm
- `publish-schemas`: Publish the schemas by creating a PR on the Abstract [schemas](https://github.com/AbstractSDK/schemas) repository.

You can see the full list of tasks available by running `just --list`.

### Testing

You can test the module using the different provided methods.

1. **Integration testing:** We provide an integration testing setup [here](./tests/integration.rs). You should use this to set up your environment and test the different execution and query entry-points of your module. Once you are satisfied with the results you can try publishing it to a real chain.
2. **Local Daemon:** Once you have confirmed that your module works as expected you can spin up a local node and deploy Abstract + your app onto the chain. You can do this by running the [test-local](./examples/test-local.rs) example, which uses a locally running juno daemon to deploy to. You can setup local juno using `just juno-local` command. At this point you can also test your front-end with the contracts.

Once testing is done you can attempt an actual deployment on test and mainnet.

### Publishing

Before attempting to publish your app you need to add your mnemonic to the `.env` file. **Don't use a mnemonic that has mainnet funds for this.**

<!-- It's also assumed that you have an account and module namespace claimed with this account before publishing. You can read how to do that [here](https://docs.abstract.money/4_get_started/5_abstract_client.html). -->

You can now use `just publish {{chain-id}}` to run the [`examples/publish.rs`](./examples/publish.rs) script. The script will publish the app to the networks that you provided. Make sure you have enough funds in your wallet on the different networks you aim to publish on.

### Generating Typescript Client Code

Before proceeding you need to install the required dependencies for the typescript client code generation.

```bash
cd packages/typescript
npm install
```

To generate the typescript client code for the contract you can run `just ts-codegen`. This will generate the code in the `typescript/src` directory. You can then import the generated code in your frontend application.

To publish the Typescript sdk, first change the name and version in the `typescript/package.json` file. Then run `just ts-publish`. This will publish the sdk to npm.

### Publishing Module Schemas

To publish your module schemas, we provide the `publish-schemas` command, which creates a pull request on the Abstract [schemas](https://github.com/AbstractSDK/schemas) repository.

```bash
just publish-schemas <namespace> <name> <version>
```

- `namespace`: Your module's namespace
- `name`: Your module's name
- `version`: Your module's version. Note that if you only include the minor version (e.g., `0.1`), you don't have to reupload the schemas for every patch version.

The command will automatically clone the Abstract Schemas repository, create a new branch with the given namespace, name, and version, and copy the schemas and metadata from your module to the appropriate directory.

For this command to work properly, please make sure that your `metadata.json` file is located at the root of your module's directory. This file is necessary for the Abstract Frontend to correctly interpret and display information about your module.

Example:

```bash
just publish-schemas my-namespace my-module 0.1
```

In the example above, `my-namespace` is the namespace, `my-module` is the module's name, and `0.1` is the minor version. If you create a patch for your module (e.g., `0.1.1`), you don't need to run `publish-schemas` again unless the schemas have changed.

## Contributing

We welcome contributions to the Abstract App Module Template! To contribute, fork this repository and submit a pull request with your changes. If you have any questions or issues, please open an issue in the repository and we will be happy to assist you.
