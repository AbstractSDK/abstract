# Abstract Accounts

Abstract Accounts are programmable smart-contract wallets used as the backbone for Abstract Apps, which will be covered later. Abstract Accounts hold funds for users and/or applications while exposing a set of programmable endpoints that can be used to configure and interact with the account.

Abstract Apps use the Abstract Account on which they are installed as the settlement layer for their transactions. In other words, Abstract Apps rarely hold funds themselves. Instead they control the funds of the Account that they are installed on. This separation of concerns allows for a more secure and modular design.

In the upcoming sections, we will delve deeper into the architecture of Abstract Accounts, providing insights into its design principles and components.

## Architecture

Abstract's infrastructure provides users with the ability to create a sovereign *smart-contract wallet*. We call this smart-contract wallet
an `Abstract Account`. The account's architecture has two primary components (smart-contracts): the **Manager** contract
and the **Proxy** contract.

```mermaid
flowchart LR
    subgraph Abstr[Abstract Account]
        direction TB
        Manager --> Proxy
    end

    Owner -.-> Manager
```

As shown in the image above, an *owner* of an account, can configure his Abstract Account by sending messages to the manager contract. We don't make any assumptions about the nature of this owner, it can be a wallet, multi-sig or any other ownership structure, allowing you to customize your ownership structure to fit your needs.

```admonish info
You can read up on the different ownership structures that we explicitly support in our [Ownership](./4_ownership.md) section.
```

The account's architecture centers around **configurable programmability**. In other words, how can one configure the account (install applications, set permissions, etc.) to enable users and developers to easily customize it to do what they want?

Let's dive deeper into the two components of the Abstract Account.

### Manager Contract

The *Manager* is responsible for the account's configuration and security, serving as the controller of the Abstract Account. It is responsible for the account's important operations, including:

- **Authentication** 🔐: Authenticating privileged calls and ensuring only approved entities can interact with the account.

- **Application Management** 📦: Managing and storing information about the applications installed on the account, their inter-dependencies, permissions and configurations.

- **Account Details** 📄: Storing the account's details, such as its name, description, and other relevant information.

### Proxy Contract

The *Proxy* is responsible for the account's programmability and assets management, serving as the asset vault of the
Abstract Account, taking care of:

- **Asset Management & Pricing** 💰: Holding the account's assets, including tokens, NFTs, and other fungible and
  non-fungible assets as well as allows for pricing assets based on decentralized exchange or oracle prices.

- **Transaction Forwarding (Proxying)** 🔀: Routing approved transactions from the **Manager** or other connected
  smart-contracts to other actors.

```admonish question
**Why are these two contracts instead of one?**

1. *Separation of concerns:* By separating the contracts the proxy's functionality (and attack surface) is as small as possible. The separation also allows for simple permission management as we want to separate the admin calls (verified by the manager) from module calls.

2. *Minimizing WASM size:* Whenever a contract is loaded for execution the whole WASM binary needs to be loaded into memory. Because all the apps proxy their messages through the Proxy contract it would be smart to have this contract be as small as possible to make it cheap to load. While CosmWasm currently has a fixed cost for loading a contract irrespective of its size. We think that might change in the future.
```

<details>
<summary>Example Interactions</summary>

### Proxy: Perform an action on Your Abstract Account

The diagram below depicts an Owner interacting with his Abstract Account through the **Manager**, and proxying a call to an external contract through the **Proxy**.

```mermaid
sequenceDiagram
    actor Owner
    participant Manager
    participant Proxy
    participant External Contract


    Owner ->> Manager: Account Action
    Manager ->> Proxy: Forward to Proxy
    Proxy ->> External Contract: Execute
```

### Manager: Enabling IBC on Your Abstract Account

Enabling the IBC functionality on your Abstract Account is done via the Manager contract with the UpdateSettings message. By doing so the IBC client will be registered to your account, enabling your modules to execute cross-chain commands.

```mermaid
sequenceDiagram
    autonumber
    actor U as Owner
    participant M as Manager
    participant VC as Version Control

    U ->> M: UpdateSettings
    Note right of U: ibc_enabled
    M -->>+ VC: Query IBC Client address
    VC -->>- M: Return IBC Client address
    M ->> M: Register IBC Client
```

</details>
