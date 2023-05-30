# Setting up the environment

Before you get started with the Abstract SDK, you will need to set up your development environment. This guide will walk you through the process of setting up your environment and creating your first Abstract app module.

## Rust

To work with the SDK you will need the Rust programming language installed on your
machine. If you don't have it installed, you can find installation instructions on [the
Rust website](https://www.rust-lang.org/tools/install).

## WASM

Additionally, you will need the Wasm compiler installed to build
WASM binaries. To install it, run:

```shell
$ rustup target add wasm32-unknown-unknown
> installing wasm32-unknown-unknown
```

## Git

You will also need git installed to clone our template repository. You can find instructions how to do so [here](https://git-scm.com/book/en/v2/Getting-Started-Installing-Git).

## Recommended tools

- [Rust Analyzer:](https://marketplace.visualstudio.com/items?itemName=rust-lang.rust-analyzer) Rust Analyzer is a language server that provides IDE support for Rust. If you use VS-Code it's highly recommended.
- [Docker:](https://docs.docker.com/desktop/) Our testing infrastructure uses Docker to run the tests. If you want to run the tests locally, you will need to install Docker.
- [Just:](https://github.com/casey/just#installation) Just is a command runner that we use to improve the development flow. You can install it by following the instructions.

## Using the Template

Time to start building! We'll get you set up with the Abstract App template which contains:

- A scaffold app module with:
  - A basic contract
  - [cw-orchestrator](https://github.com/AbstractSDK/cw-orchestrator) interface and deployment script
  - Integration tests
- A scaffold front-end with:
  - A basic UI
  - A basic API client
- A set of just commands that will boost your productivity.


Go to [our App Template on Github](https://github.com/AbstractSDK/app-template) and click on the "Use this template" button to create a new repository based on the template. You can name the repository whatever you want, but we recommend using the name of your module.

Go ahead and read through the readme of the template repository. It contains instructions on how to set up your development environment and how the template is structured.

## Tools used in the template

The following Rust tools are used extensively in our template to improve your productivity.

- [Taplo:](https://taplo.tamasfe.dev/cli/installation/cargo.html) The CI shipped with the template will perform formatting checks. To ensure you pass the checks, you can install Taplo and use the `just format` command to format your code and toml files.
- [Nextest:](https://nexte.st/index.html) A better cargo test runner.
- [Cargo Limit:](https://github.com/alopatindev/cargo-limit) Prioritizes errors over warnings in compile output as well as some other small improvements.
- [Cargo Watch:](https://crates.io/crates/cargo-watch) Allows you to automatically re-run compilation when files change. This is useful when you are working on the contracts and want to fix compiler errors one by one.

You can install them by running `just install-tools`. All of the tools are built from source by Cargo.

## Contract layout

The template contains a scaffold contract that you can use as a starting point for your own contract. The contract is located in the `src` directory and is structured as follows:

- `contract.rs`: This file is the top-level file for your module. It contains the type definition of you module and the const builder that constructs your contract. It also contains a macro that exports your contract's entry points. You can also specify the contract's dependencies here.
- `error.rs`: This file contains the error types that your contract can return.
- `msg.rs`: This file contains the custom message types that your contract can receive. These messages also have `cw-orchestrator` macros attached to them which comes in useful when you are writing your integration tests.
- `state.rs`: This file contains the state types that your contract will use to store state to the blockchain.
- `interface.rs`: This file contains the interface that your contract will use to interact with the `cw-orchestrator` library.
- `replies/`: This directory contains the reply handlers that your contract will use to handle replies.
- `handlers/`: This directory contains the message handlers that your contract will use to handle the different messages it can receive.

## Front-end layout

TODO: 