![Abstract Nightly Build](https://github.com/AbstractSDK/abstract/actions/workflows/nightly.yml/badge.svg)

# Abstract

Monerepo for the [Abstract Money](https://abstract.money) project.

![banner](https://raw.githubusercontent.com/AbstractSDK/assets/mainline/v2/Logo_Banner.jpeg)

Abstract is a CosmWasm development platform designed to empower developers to craft secure and dynamic applications effortlessly. It boasts a modular architecture and provides an exhaustive range of tools, streamlining the development process and catalyzing the innovation of solutions in the blockchain space.
The Abstract SDK is a Rust library for on-chain operations, cw-orchestrator aids in smart contract deployment, and Abstract JS enables browser-based smart contract interactions.

## Quick Start

Checkout [this tutorial](https://docs.abstract.money/4_get_started/1_index.html) to quickly get started with Abstract SDK. Dive into our [Frequently Asked Questions](https://docs.abstract.money/video_and_content/faq.html) for everything Abstract related.

You can find [Prerequisites](https://docs.abstract.money/3_framework/0_technologies.html) that you need to get started with Abstract.

Read [the complete documentation](https://docs.abstract.money/) for more information on each product and how to integrate them.

## Repository Structure

The repository is structured as follows:

- **app-template:** A template for creating new Abstract apps. See [app-template repo](https://github.com/AbstractSDK/app-template) for a cloneable version.
- **framework:** The Abstract framework. This is the core of the Abstract project and contains the smart-contracts and packages related to the on-chain abstract accounts and surrounding infrastructure. Have a look at the [Abstract SDK documentation](https://docs.abstract.money/3_framework/1_abstract_sdk.html) if you're interested in building with Abstract!
- **modules:** The Abstract-maintained modules. These are a set of example apps that are built on top of the Abstract framework. They can be used as standalone applications or as building blocks for creating more complex applications.
- **integrations:** Integrations with other projects, particularly dexes.
- **scripts:** Scripts for CI.
- **interchain:** Contains a testing suite for Abstract Interchain Accounts as well as scripts to deploy the framework to the interchain.

## Contributing

See [CONTRIBUTING](https://docs.abstract.money/contributing.html) for details on how to contribute.
If you want to follow the updates or learn more about the latest design then join our [Discord](https://discord.com/invite/uch3Tq3aym).
