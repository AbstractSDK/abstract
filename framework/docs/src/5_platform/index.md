# Contracts

```admonish info
At this point you have enough knowledge to start building your own Abstract Module. If you want to start building, head over to our [Getting Started](../4_get_started/1_index.md) section! 🛠️
```

In the previous sections, we covered different high-level aspects of the Abstract framework. In the following sections, we will explore the different contracts of the Abstract framework in more detail.

Here's a peek into the key elements that form the foundation of the Abstract framework:

- [Abstract Name Service (ANS)](./1_ans.md): A smart-contract oriented name service that enables chain-agnostic action execution
  by storing easily retrievable address related data on the blockchain.

- [Version Control](./2_version_control.md): A comprehensive on-chain registry for accounts and modules.
  It exposes namespace claiming, module
  registrations, and detailed querying of modules by namespace, name, and version.

- [Account Factory](./3_account_factory.md): Allows for the creation of (Interchain) Abstract Accounts, detailed in
  the section on [Interchain Abstract Accounts](../3_framework/8_ibc.md).

- [Account Console](./4_account_console.md): A web-based developer-oriented interface designed to simplifying managing and
  interacting with your Accounts. Access it here: [console.abstract.money](https://console.abstract.money).

- [Module Factory](./5_module_factory.md): Facilitates installing Abstract Modules on an Account.
  You can install modules by interacting with the Account Manager directly, i.e. via CLI/scripts, or by using the Account Console.

## Features

Through the interplay of the components above, Abstract offers a number of features that make it a powerful framework for sustainable application development.

- [Monetization](./6_monetization.md): Developers have the ability to monetize their modules by setting
  installation fee or usage fees for
  their modules. By providing direct monetization strategies we aim to reduce funding intermediaries and improved the sustainability of small team/solo developer projects.
  
- [Account Value Oracle](./7_oracle.md): An integrated way to easily get the value of your Account's assets **on-chain**.
