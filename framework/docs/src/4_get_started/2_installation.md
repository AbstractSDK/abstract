# Setting up the environment

Before you get started with the Abstract SDK, you will need to set up your development environment. This guide will walk you through the process of doing just that.

```admonish info
Experienced with CosmWasm? Skip to the [Using The Template](#using-the-template) section.
```

## Rust

To work with the SDK you will need a Rust toolchain installed on your machine. If you don't have it installed, you can find installation instructions on <a href="https://www.rust-lang.org/tools/install" target="_blank">the official Rust website</a>.

## WASM

Additionally, you will need the WASM compile target installed to build WASM binaries. You will need `rustup`, which you got when installing Rust on the previous step. To install it the WASM compile target, run:

```shell
$ rustup target add wasm32-unknown-unknown
> installing wasm32-unknown-unknown
```

## Docker

<a href= "https://www.docker.com/" target="_blank">Docker</a> is used to create a containerized environment for facilitating reproducible builds. Specifically we'll be using a slightly modified builder image available <a href= "https://hub.docker.com/r/abstractmoney/workspace-optimizer" target="_blank">here</a>. You can read about the slight modifications we made in <a href= "https://github.com/CosmWasm/rust-optimizer/pull/130" target="_blank">this PR</a>.

## Git

You will also need `git` installed to clone our template repository. You can find instructions for installing `git` on your operative system <a href="https://git-scm.com/book/en/v2/Getting-Started-Installing-Git" target="_blank">here</a>.

## Using the Template

Now we'll get you set up with the Abstract App template which contains:

- A scaffold app module with:
  - A basic contract
  - [cw-orchestrator](../1_products/1_cw_orchestrator.md) interface and deployment script
  - Integration tests
- A set of just commands that will help you in your development.

Go to <a href="https://github.com/AbstractSDK/app-template" target="_blank">our App Template on Github</a> and click on the "Use this template" button to create a new repository based on the template. You can name the repository whatever you want, but we recommend using the name of your module.

![](../resources/get_started/use-this-template.webp)

```admonish success
To quickly get started, run `./template_setup.sh` and install the recommended tools.
```

Go ahead and read through the readme of the template repository to learn how it is structured. It contains instructions on how to set up your development environment, useful commands you can perform using `just`, how to test and deploy your app, and more.

## Contract file structure

The template contains a scaffold contract that you can use as a starting point for your own contract. The contract is located in the `src` directory and is structured as follows:

- `contract.rs`: Top-level file for your module. It contains the type definition of you module and the const builder that constructs your contract. It also contains a macro that exports your contract's entry points. You can also specify the contract's dependencies here.
- `error.rs`: Error types that your contract can return.
- `msg.rs`: Custom message types that your contract can receive. These messages also have `cw-orchestrator` macros attached to them which comes in useful when you are writing your integration tests.
- `state.rs`: State types that your contract will use to store state to the blockchain.
- `interface.rs`: Interface that your contract will use to interact with the `cw-orchestrator`
  library.
- `replies/`: Reply handlers that your contract will use to handle replies.
- `handlers/`: Message handlers that your contract will use to handle the different messages it can receive.

If there's anything you don't understand about the template please don't hesitate to reach out to us on our <a href="https://discord.com/invite/uch3Tq3aym" target="_blank">Discord</a> server.
