# Abstract SDK

Modular interchain application framework to enable the next generation of onchain apps. See [Abstract Money](https://abstract.money) if you're looking for the user-facing application platform.

[High-level overview](https://medium.com/abstract-money/a-high-level-overview-of-abstract-moneys-interchain-app-platform-d30813c7a1b4) • [Latest 0.22 Release log](https://medium.com/abstract-money/abstract-v0-22-simplifying-the-ibc-experience-77f06a324c4d).



![banner](https://raw.githubusercontent.com/AbstractSDK/assets/mainline/v2/Logo_Banner.jpeg)




The Abstract SDK is the leading CosmWasm application framework designed for secure, composable, and interoperable applications. The framework allows developers to write CosmWasm modules that work together to form applications, akin to microservice architectures in Web2. These modules are installable and controlled by a smart-contract wallet called an Abstract Account, the on-chain component to our application platform. Other on-chain services on the Abstract platform include a module registry, where developers can list and monetize their code; the Abstract Name Service (ANS) for registering and resolving assets, contracts, channels, and dex pools; and our IBC infrastructure, enabling chain abstraction through Interchain Abstract Accounts. The Abstract platform is deployable on any CosmWasm virtual machine, and is best used alongside [cw-orchestrator](https://orchestrator.abstract.money), our powerful, Rust-based, CosmWasm scripting testing and deployment tool.

If you're looking for some awesome projects built on Abstract, definitely check out [Awesome Abstract](https://github.com/AbstractSDK/awesome-abstract) for some highlights.

Abstract is available on every major CosmWasm chain including Osmosis, Neutron, Archway, Kujira, Terra, Injective, Juno, XION, and compatible with Celestia Rollkit, Strangelove Rollchains and many more.

## Quick Start

Check out [this tutorial](https://docs.abstract.money/get_started/index.html) to quickly get started with Abstract SDK. Dive into our [Frequently Asked Questions](https://docs.abstract.money/video_and_content/faq.html) for everything Abstract related.

You can find [Prerequisites](https://docs.abstract.money/framework/0_technologies.html) that you need to get started with Abstract.

Read [the complete documentation](https://docs.abstract.money/) for more information on each product and how to integrate them.

## Repository Structure

The repository is structured as follows:

- **app-template:** A template for creating new Abstract apps. See [app-template repo](https://github.com/AbstractSDK/app-template) for a cloneable version.
- **framework:** The Abstract framework. This is the core of the Abstract project and contains the smart-contracts and packages related to the on-chain abstract accounts and surrounding infrastructure. Have a look at the [Abstract SDK documentation](https://docs.abstract.money/framework/abstract_sdk.html) if you're interested in building with Abstract!
- **modules:** The Abstract-maintained modules. These are a set of example apps that are built on top of the Abstract framework. They can be used as standalone applications or as building blocks for creating more complex applications.
- **integrations:** Integrations with other projects, particularly dexes.
- **scripts:** Scripts for CI.
- **interchain:** Contains a testing suite for Abstract Interchain Accounts as well as scripts to deploy the framework to the interchain.

## Contributing

See [CONTRIBUTING](https://docs.abstract.money/contributing.html) for details on how to contribute.
If you want to follow the updates or learn more about the latest design then join our [Discord](https://discord.com/invite/uch3Tq3aym).

PS. If you've made it down here, we're expanding our platform to allow Abstract Accounts to control any chain over IBC, including Ethereum (and other EVM chains), Solana, Avalanche, and more. Reach out to us, we're hiring, and we'd love to have you on the team.
