# Modularity in Abstract

Modularity is a fundamental design principle of the Abstract platform, contributing significantly to its flexibility and extensibility. A module in Abstract is a self-contained unit of functionality that can be added to an Abstract Account to extend its capabilities. This modular approach promotes the construction of bespoke applications, enhances the scalability of the system, and improves overall maintainability.

For application developers, modules simplify the development process by offering pre-built functionalities. Instead of building every aspect from scratch, developers can leverage modules either from Abstract's extensive library or those crafted by other developers available in the module marketplace. Additionally, developers have the freedom to create custom modules using the [Abstract SDK](https://docs.rs/abstract-sdk/latest/abstract_sdk), tailoring them to specific application needs and even contributing back to the community.

## How Modules Work

In Abstract, the process of integrating these modules is managed through the Manager contract within an Abstract Account. The Manager keeps track of all installed modules, managing their permissions and interactions. This system enables the customization of each Account, as modules can be installed or uninstalled as per the user's requirements, thereby adjusting the Account's functionality.

From the perspective of a developer, the Abstract framework sets conventions and standards that allow leveraging existing modules during the development of new ones. This layer of abstraction saves considerable time and effort while promoting consistent design across different modules.

## Security

Security is a priority at Abstract, especially when it comes to the modules that extend the capabilities of an Abstract Account. Every module listed on the mainnet marketplaces must undergo a thorough auditing process before it's made available to users. This process scrutinizes the module's code, checking for potential vulnerabilities, and ensuring that it adheres to best security practices.

While no system can guarantee absolute security, this rigorous vetting process, coupled with the inherent security measures in Abstract and CosmWasm's architecture, mitigates potential risks to a considerable extent.

The Abstract platform also maintains a Version Control (TODO link) for all the modules, allowing users and developers to track changes, understand the evolution of a module, and choose versions that have passed security audits. 



TODO: add diagram

------
TODO: below is not reviewed

## How to Use Modules in Your Project
Leveraging modules in your project can significantly simplify the development process, allowing you to deploy projects faster and more cost-effectively than building traditionally. By using pre-built, tested, and community-reviewed modules, you can focus on your project's unique features while reducing development time and minimizing potential bugs.

Here's an example of how modules can be leveraged for a decentralized finance (DeFi) project:

Imagine you want to create a DeFi application with the following features:

- A fungible token for your platform
- Staking and delegation functionality
- A governance system for community-driven decision-making
- Integration with an oracle for fetching off-chain data

Instead of building each feature from scratch, you can leverage Abstract's off-the-shelf modules to implement these functionalities with ease. This not only saves time and resources but also ensures that your project benefits from the best practices established by the Abstract community.

**Step 1:** Choose the relevant modules from the Abstract library, such as:
Token Module for fungible tokens
Staking Module for staking and delegation
Governance Module for creating a governance system
Oracle Module for integrating with an oracle service

**Step 2:** Import the chosen modules into your project and configure them according to your requirements. This can include setting custom parameters, such as token supply, staking rewards, or voting thresholds.

**Step 3:** Integrate the modules with your existing codebase, ensuring they work seamlessly with your project's unique features. This can involve calling module functions, implementing hooks, or extending your data structures.

**Step 4:** Test your dApp thoroughly to ensure the modules function as intended and do not introduce any unexpected behavior.

By leveraging Abstract's modules in this way, you can rapidly build and deploy your DeFi project while benefiting from the robustness and flexibility of the Abstract ecosystem.
