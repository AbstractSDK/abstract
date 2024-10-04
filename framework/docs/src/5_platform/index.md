# Abstract Infrastructure

```admonish info
At this point you have enough knowledge to start building your own Abstract Module. If you want to start building, head over to our [Getting Started](../4_get_started/1_index.md) section! 🛠️
```

In the previous sections, we covered different high-level aspects of the Abstract framework. In the following sections, we will outline the different contracts of the Abstract infrastructure in more detail.

## On-Chain Contracts

- [Module Factory](./5_module_factory.md): Facilitates installing Abstract Modules on an Account.

- [Abstract Name Service (ANS)](./1_ans.md): A name service that enables chain-agnostic action execution
  by storing commonly retrieved data such as assets, contracts, and IBC channels.

- [Version Control](./2_registry.md): A registry for modules and accounts.
  It exposes namespace claiming, module registrations, and detailed querying of modules by namespace, name, and version.

## Features

Through the interplay of the components above, Abstract offers a number of features that make it a powerful framework for sustainable application development.

- [Monetization](./6_monetization.md): Developers have the ability to monetize their modules by setting
  installation fee or usage fees for
  their modules. By providing direct monetization strategies we aim to reduce funding intermediaries and improved the sustainability of small team/solo developer projects.
