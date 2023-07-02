<!-- # The Abstract Development Platform -->
<!-- This page is a high-level overview of the Abstract Platform, not the SDK or any of our actual products. Hence we should stick to a vocabulary that is familiar to the reader. -->
![SDK Background](https://github.com/AbstractSDK/assets/blob/mainline/twitter_cover-1.png?raw=true)
<!-- 
<div align="center">

### Welcome to Abstract, the interchain CosmWasm development platform. </br>We are building tools and infrastructure for composable smart-contract applications. 

</div> -->
## What Is The Abstract Platform?

The Abstract platform provides a combination of [CosmWasm][1]-oriented products.

- On-chain smart-contract infrastructure ([Abstract SDK](#the-abstract-sdk))
- Development tooling ([cw-orchestrator](#cw-orchestrator))
- Front-end libraries ([Abstract.js](#abstractjs))

Our products are designed to be composable, allowing developers to re-use the components they need to build their applications. While Abstract aims to simplify the development experience, it functions as a powerful tool, enabling you to innovate with less effort.

```admonish info
In this page you are introduced to the Abstract platform and its components. You can skip the introduction and go straight to the [Getting Started](./4_get_started/1_index.md) guide if you're already familiar with the platform.
```

### The Abstract SDK

The Abstract SDK is a modular smart-contract framework designed to enhance the development of decentralized applications. The SDK is built on top of [CosmWasm][1]: a battle-tested WASM-based smart-contract framework, written in [Rust][2]. The Abstract SDK is comprised of two main components:

#### Abstract Accounts

An Abstract Account is a smart-contract wallet developed by Abstract. That means that the smart-contract is capable of holding tokens and interacting with other smart-contracts. The contract's ownership structure is customizable to fit the needs of users or applications. It is designed to be highly programmable, allowing developers to build and distribute complex applications on top of it.

#### Abstract Apps

Abstract Apps are smart-contracts that add functionality to an Abstract Account. Here's a small snippet of code to give you an idea of how an App is created with the Abstract SDK:

```rust,no_run
{{#include ../../packages/abstract-app/examples/counter.rs:handlers}}
```

The code above defines an **Abstract App**. This app can be installed on any Abstract Account through the Abstract App store, allowing developers to monetize their code.

The customizable handlers that are used in the builder are functions similar to the native CosmWasm entry-point functions. They expose an additional App object which, via the `abstract-sdk`, empowers you to execute intricate multi-contract transactions with minimum code. Importantly, this simplification does not limit the contract's programmability. Instead, it provides a balance of efficient coding and comprehensive control over inter-contract interactions.

```admonish info
Already familiar with [cw-orchestrator](https://github.com/AbstractSDK/cw-orchestrator)? Skip to [the SDK's introduction page.](./3_framework/1_abstract_sdk.md)
```

### Cw-Orchestrator

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

Using cw-orchestrator for your smart-contract interactions reduces your testing/deployment overhead and improves both the code's readability and maintainability.

## Abstract.js

Abstract.js is the Javascript package for interacting with the on-chain Abstract framework. More documentation will be added soon.

[1]: https://cosmwasm.com/
[2]: https://www.rust-lang.org/

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
