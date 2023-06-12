# The Abstract Development Platform

<!-- This page is a high-level overview of the Abstract Platform, not the SDK or any of our actual products. Hence we should stick to a vocabulary that is familiar to the reader. -->
![SDK Background](./assets/docs_bg.png)

<div align="center">

### Welcome to Abstract, the interchain CosmWasm development platform. We are building tools and infrastructure for the next generation of smart-contract applications.

</div>

## What Is The Abstract Platform?

The Abstract platform is a combination of CosmWasm tooling, on-chain smart-contract infrastructure and front-end libraries. It's designed to be modular, allowing developers to choose the components they need to build their applications. While Abstract tries to simplify a lot of the development experience, it is inherently a power-tool, allowing you to get more done with less effort.

### The Abstract SDK

The Abstract SDK is a modular smart-contract framework designed to enhance the development of CosmWasm dApps. It's built on top of CosmWasm; a battle-tested WASM-based smart-contract framework, written in Rust.

Here's a small snippet of code to give you an idea of what using the Abstract SDK looks like:

```rust,no_run
{{#include ../../packages/abstract-app/examples/counter.rs:handlers}}
```

The code above defines an Abstract app, which is a smart-contract that can be installed on any Abstract Account. The app has a unique ID and a version, both are used to offer the app on the Abstract App store, allowing other users to install the app on their account.

The customizable handlers allow for full developer freedom and have an App object available within them. This object is highly programmable through the [abstract-sdk](./get_started/sdk.md), which is our Account Abstraction Toolbox. In a nutshell it allows you to perform complex multi-contract interactions with very minimal code while not compromising on the contract's programmability in any way.

> Looking to get started? Check out the [Getting Started](./3_get_started/1_index.md) guide.

### Cw-Orchestrator

[cw-orchestrator](https://github.com/AbstractSDK/cw-orchestrator) is a smart-contract scripting library that allows you to re-use code for use in testing and deployments. It is also our primary tool for making Abstract's infrastructure highly available allowing us to make developing with Abstract a pleasant experience.

Here's a snippet that sets up the complete Abstract framework on a cw-multi-test environment and deploys the previously shown App contract to the framework.

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
    .claim_namespaces(1, vec!["my-namespace".to_string()])?;

// Deploy the app!
contract.deploy(APP_VERSION.parse()?)?;
```

Using cw-orchestrator for your testing and deployment needs reduces your testing setup and complexity and improves both the code's readability and maintainability. Because of its design it allows us to share our contracts with any developer that wants to use them with a few lines of code. This greatly reduces the friction of using our code and it allows you to provide the same experience to developers that might want to use your code, giving you an edge over other options.

## Abstract.js

Abstract.js is the Javascript package for interacting with the on-chain Abstract framework.

<!-- ## Features

- Chain-Agnostic: The Abstract platform is built to support various blockchain networks within the Cosmos ecosystem, giving developers the flexibility to choose the best-suited platform for their dApp.

- Modular Design: Abstract's modular architecture allows developers to choose from a curated library of smart-contract or develop their own using the Abstract SDK, enabling rapid development and customization.

- Custom Governance: Abstract allows developers to create custom governance structures tailored to their application's specific requirements, including integration with multi-signature wallets and DAOs.

- Development Tooling [cw-orchestrator](https://github.com/AbstractSDK/cw-orchestrator): Abstract offers a vast integration testing library, enabling developers to quickly increase test coverage and ensure the reliability of their dApps.

- Version Management: Abstract simplifies the process of managing smart-contract versioning, ensuring developers can quickly release and access new features and bug-fixes.

- Ecosystem Integrations: Abstract is designed to work seamlessly with popular projects and services in the Cosmos ecosystem, such as:

  - Dexes: WyndDex, Osmosis, Astroport, and TerraSwap
  - Oracles: Ojo Network (planned)
  - Wallets: Keplr, Cosmostation and Terra Station
  - Automation: CronCat and Lit Protocol (planned) -->

<!-- ## Use Cases and Examples

### Personalized Decentralized Finance (PDeFi)

Developers can leverage Abstract's modular architecture to quickly build Personalized DeFi applications. In contrast with traditionally DeFi, these applications aren't required to custody the user's assets. They can also be customized to meet the user's specific requirements, such as integrating with their preferred oracle, wallet, or exchange. Abstract's P-DeFi applications can be used to create custom financial products, such as:

- Dollar cost averaging (DCA)
- Grid bot trading
- Custom trading strategies
- Permissioned asset markets

All of which can have tweakable settings that are in the user's control.

### NFT Marketplaces

Abstract's SDK can be used to create customizable NFT marketplaces, allowing users to mint, trade, and manage digital assets on various Cosmos-based blockchains.

### Governance Platforms

Developers can utilize Abstract to create advanced governance platforms that support custom voting mechanisms and proposal systems for community-driven decision-making.

### Cross-Chain Interoperability

Abstract's chain-agnostic nature enables developers to create dApps that can interact with multiple blockchains within the Cosmos ecosystem, facilitating cross-chain asset transfers and communication.

### Decentralized Autonomous Organizations (DAOs)

Abstract can be used to build and manage DAOs with custom governance structures, allowing communities to collaborate, make decisions, and allocate resources effectively.

### Project Infrastructure

Develop tooling for your dApps that help speed up work flows, work with your contracts from inside Rust, no need for JavaScript, use your own Rust projects codebases.

 -->
