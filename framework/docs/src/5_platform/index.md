# Framework Components

```admonish info
At this point you have enough knowledge to start building your own Abstract Module. If you want to start building, head over to our [Getting Started](../4_get_started/1_index.md) section! 🛠️
```

In the previous sections, we covered different aspects of the Abstract framework on a higher level. In this section, we
will explore the different components under the hood of the Abstract framework that makes it powerful and
unique, enabling it to work the way it does.

Here's a peek into the key elements that form the foundation of the Abstract framework:

- [Abstract Name Service (ANS)](./1_ans.md): Enables chain-agnostic action execution by storing crucial address
  space-related data on the deployed blockchain which can easily be referenced.

- [Version Control](./2_version_control.md): Acts as a comprehensive on-chain registry for accounts and software.
  It exposes namespace claiming, module
  registrations, and seamlessly querying modules by namespace, name, and version.

- [Account Factory](./3_account_factory.md): Allows for the creation (Interchain) Abstract Accounts, which can be
  interacted with via scripts or the <a href="https://app.abstract.money" >Account Console web interface</a>.

- [Account Console](./4_account_console.md): A web-based interface designed for optimal interaction with your Abstract
  Accounts. It's a powerful tool that contains all the features you need not only to manage your accounts but also help
  you develop your application.

- [Module Factory](./5_module_factory.md): Allows you to install and manage Abstract Modules via the Account Manager.
  You
  can install modules by interacting with the Account Manager directly, i.e. via CLI/scripts, or by using the Account Console.

- [Monetization](./6_monetization.md): Developers have the potential to monetize their modules by setting an
  installation fee for
  others to use their modules. By introducing monetization strategies, Abstract offers developers
  incentives to build and share valuable modules with the community.
  
- [Value Oracle](./7_oracle.md): An integrated way to get the value of your account's assets **on-chain**.

In the following pages we will explore each of these components in detail. If you have any questions, please don't
hesitate to reach out to us on <a href="https://discord.com/invite/uch3Tq3aym" target="_blank">Discord</a>.
