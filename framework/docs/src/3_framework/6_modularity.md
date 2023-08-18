# Modularity in Abstract

Modularity is a fundamental design principle of the Abstract platform, contributing significantly to its flexibility and
extensibility. A module in Abstract is a self-contained unit of functionality that can be added to an Abstract Account
to extend its capabilities. This modular approach promotes the construction of bespoke applications, enhances the
scalability of the system, and improves overall maintainability.

For application developers, modules simplify the development process by offering pre-built functionalities. Instead of
building every aspect from scratch, you can leverage modules either from Abstract's extensive library or those
crafted by other developers available in the module marketplace. Additionally, you have the freedom to create
custom modules using the [Abstract SDK](https://docs.rs/abstract-sdk/latest/abstract_sdk), tailoring them to specific
application needs and even contributing back to the community.

```admonish info
Skip to [Modules](./7_module_types.md) to learn what the kinds of modules are and why they are relevant to your Abstract Account.
```

## How Modules Work

In Abstract, the process of integrating these modules is managed through the Manager contract within an Abstract
Account. As described in a [previous section](4_architecture.md), the Manager keeps track of all installed modules,
managing their permissions and interactions. This system facilitates the customization of individual Abstract
Accounts, permitting the installation or removal of modules according to your needs. In doing so, it effectively
adapts the Account's functionality.

From the perspective of a developer, the Abstract framework sets conventions and standards that allow leveraging
existing modules during the development of new ones. This layer of abstraction saves considerable time and effort while
promoting consistent design across different modules.

## Security

Security is a priority at Abstract, especially when it comes to the modules that extend the capabilities of an Abstract
Account. Every module listed on the mainnet marketplaces must undergo a thorough auditing process before it's made
available to developers. This process scrutinizes the module's code, checking for potential vulnerabilities, and
ensuring that it adheres to best security practices.

While no system can guarantee absolute security, this rigorous vetting process, coupled with the inherent security
measures in Abstract and CosmWasm's architecture, mitigates potential risks to a considerable extent.

The Abstract platform also maintains a [Version Control](../5_platform/version_control.md) for all the modules, allowing
developers to track changes, understand the evolution of a module, and choose versions that have passed security audits. 
