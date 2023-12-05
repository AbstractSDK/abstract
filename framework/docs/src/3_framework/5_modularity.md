# Modules

Modularity is a fundamental design principle of the Abstract platform, contributing significantly to its flexibility and
extensibility. A module in Abstract is a self-contained smart-contract that can be added to an Abstract Account
to extend its capabilities. This modular approach promotes the construction of complex applications, enhances the
scalability of these applications, and improves overall maintainability and security.

For application developers, modules simplify the development process by offering pre-built functionalities. Instead of
building every part of an application from scratch, you can leverage modules either from Abstract's extensive library or those
crafted by other developers available in the module marketplace. Additionally, you have the freedom to create
custom modules using the [Abstract SDK](https://docs.rs/abstract-sdk/latest/abstract_sdk), tailoring them to specific
application needs and contributing back to the community.

## How Modules Work

A module's functionality can be accessed by installing it on an Abstract Account. The process of installing a module is managed by the Manager contract of the Abstract
Account on which the module is being installed. As described in a [previous section](3_architecture.md), the Manager keeps track of all installed modules and managing their permissions and interactions.

Through this system users can easily customize individual Abstract
Accounts, permitting the installation or removal of modules according to their needs. In doing so, it effectively
adapts the Account's functionality.

From the perspective of a developer, the Abstract framework sets conventions and standards that allow leveraging
existing modules during the development of new ones.

## Security

Security is a priority at Abstract, especially when it comes to the modules that extend the capabilities of an Abstract
Account. Every module listed on the mainnet marketplaces must undergo a thorough auditing process before it's made
available to users and developers. This process scrutinizes the module's code, checking for potential vulnerabilities, and
ensuring that it adheres to best security practices.

While no system can guarantee absolute security, this rigorous vetting process, coupled with the inherent security
benefits of Abstract's and CosmWasm's architecture, mitigates potential risks to a considerable extent.

The Abstract platform also maintains a [Version Control](../5_platform/2_version_control.md) for all the modules, allowing
developers to track changes, understand the evolution of a module, and choose versions that have passed security audits.
