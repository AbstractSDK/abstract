# Abstract SDK - How to get started

![SDK Background](../resources/abstract_platform/twitter_cover-1.png)

Welcome to the builder section of the Abstract documentation. The following sections will walk you through the process of setting up your development environment, creating an [App Module](../3_framework/6_module_types.md#apps) and deploying it to our on-chain registry so that it can be used by others.

```admonish info
Coming from 👾EVM👾 ? Be sure to read up on CosmWasm and its differences from EVM in the [CosmWasm](./3_framework/0_technologies.md#cosmwasm) section.
```

## Tools used in this guide

Here are the most important tools you will need to know about to get started with the Abstract SDK:

1. A minimal understanding of Rust is expected. If you are new to Rust, you can find a great introduction to the language in the <a href="https://doc.rust-lang.org/book/" target="_blank">The Rust Book</a>.

2. The Abstract SDK is built using the CosmWasm smart-contract framework. If you are new to CosmWasm, you can find a great introduction to the framework in the <a href="https://book.cosmwasm.com/" target="_blank">CosmWasm Book</a>.

3. Abstract also makes extensive use of [cw-orchestrator](../1_products/1_cw_orchestrator.md), our CosmWasm scripting library. You can read its documentation <a href="https://orchestrator.abstract.money/" target="_blank">here</a>.

## Recommended tools

- <a href="https://marketplace.visualstudio.com/items?itemName=rust-lang.rust-analyzer" target="_blank">Rust Analyzer</a>: Rust Analyzer is a language server that provides IDE support for Rust. If you use VS-Code it's highly recommended.
- <a href="https://plugins.jetbrains.com/plugin/8182-rust" target="_blank">Intellij Rust Plugin</a>: open-source Rust plugin compatible with all IntelliJ-based IDEs. You are going to need it if you are using the Intellij IDEA Community Edition IDE, however it's not needed for the Rust Rover.
- <a href="https://github.com/casey/just#installation" target="_blank">Just</a>: Just is a command runner that we use to improve the development flow. You can install it by following the instructions on the Github repository.
