# Modules

Modularity is a core design principle of the Abstract platform to allow for more maintainable and composable applications.

A *module* in Abstract is a smart-contract that can be installed on an Abstract Account to extend the account's capabilities. Modules can be installed, removed, and configured by the account's owner, allowing for a high degree of customization. Additionally, modules can take on dependencies on other modules and securely interact with each other.

Instead of building every part of an application from scratch, application developers can make use of an existing availability of pre-built logical components.

```admonish question
When you're writing software, do you use libraries or do you write everything from scratch?

The obvious answer is that you use libraries. You use libraries because they save you time and effort, allowing you to focus on the core functionality of your application. You also use libraries because they're tested and proven to work, reducing the risk of bugs and vulnerabilities.

Modules allow you to do the same thing, but with smart-contracts.
```

You can leverage modules either from Abstract's extensive library or those crafted by other developers available in the module library. Most importantly any application on Abstract, including yours, is a module.

## How Modules Work

As mentioned, a module's functionality can be accessed by installing it on an Abstract Account. The process of installing a module involves calling the Abstract
Account to perform its installation. An analogy to installing a module is pressing the "install" button for any app in the iOS/Android app store where the module is the App and the Abstract Account is the phone.

```admonish info
As described in a [previous section](3_architecture.md), the Account keeps track of all installed modules and manages their permissions and interactions.
```

Through this system users can easily customize individual Abstract Accounts, permitting the installation or removal of modules according to their needs. In doing so, it effectively adapts the Account's functionality.

From the perspective of a developer, the Abstract framework sets conventions and standards that allow leveraging existing modules during the development of new ones. It also provides a supply chain for smart-contract software, allowing developers to create and market their modules to users and other developers through our platform.

## Security

Security is a top priority at Abstract. Every module listed on the mainnet marketplaces must undergo a thorough auditing process before it is made available to users and developers. This process scrutinizes the module's code, checking for potential vulnerabilities, and ensuring that it adheres to best security practices. Additionally we are firm believers of open-source software and encourage developers to publish their code on GitHub.

While no system can guarantee absolute security, this rigorous vetting process, coupled with the inherent security
benefits of Abstract's and CosmWasm's architecture, mitigates potential risks to a considerable extent.

The Abstract platform also maintains a [Registry](../5_platform/2_registry.md) for all the modules, allowing
developers to track changes, understand the evolution of a module, and choose versions that have passed security audits.
