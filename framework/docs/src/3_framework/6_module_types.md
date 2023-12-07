# Module Types

As explained in the previous section, a _module_ is a smart-contract that **extends an Account's functionality**. You can explore
all the available modules on the modules tab of your Account through
the <a href="https://app.abstract.money/" target="_blank">web-app</a>.

```admonish info
In the previous sections we referred to these modules as "applications". We did this to simplify the mental framework of 
the Abstract SDK. However, the term "application" is not entirely accurate, instead we should refer to them as "modules". 
These modules come in different types, each with their own purpose. The most common of which is the "App" module, or 
"Abstract App".
```

Modules are classified in the following categories:

- **App**: modules that add a functionality, exposing new entry-points for you or your users.
- **Adapter**: modules that act as a standard interface between your Account and external services.
- **Standalone**: modules not built within Abstract, but registered to your Account so that you Account can manage them.

<img src="../assets/modules.svg" alt="drawing" width="400" style="vertical-align:middle"/>

## Module IDs

Every module is uniquely identified by a module ID. This ID is a string that follows the following format:

```text
<namespace>:<name>
```

The namespace is a string that resembles the publishing domain of a module developer, while the name is the name of the module itself. For example, the `abstract:etf` module is an App module developed by Abstract where `abstract` is the namespace and `etf` is the name of the module.

Additionally each module has a SEMVER version number that can be used to uniquely identify a specific version of a module, or just get the latest version. Module IDs and their versions are used to install modules on an Abstract Account.

A module ID is independent of the kind of module it refers to.

## Apps

An App module adds or alters the functionality of an Abstract Account, exposing new functions to you and/or your users.
This could range from adding advanced financial logic to data management features or permission systems,
depending on your use-case.

Each App module instance is exclusive to a single Abstract Account, **meaning the instance is created and owned by the Account**,
ensuring the owner has full control over the module's functionality and lifecycle. This level of control extends to the management of upgrades, maintenance, and any customization that might be required for the specific use case of the application.

Because each Account has its own instance of an App, App modules can be tightly integrated with the Account's existing infrastructure. This includes the ability to interact directly with other modules (including Apps) installed on the same account, enabling powerful synergies and cross-module functionality.

```admonish example
The `abstract:etf` module is an app that allows external users to buy and sell "shares" in your Account, representing a portion of the Accounts' value.
```

## Adapters

Adapters serve as standard interfaces that facilitate communication between your Abstract Account and various external services. They act like bridges, enabling your account to interact with different smart contracts and blockchain services, thereby enhancing the interoperability of your applications.

The key function of an Adapter is to generalize functionality. Regardless of the underlying blockchain or smart contract protocol, the Adapter provides a standard interface that maintains consistency and simplifies the interaction process. As such, Adapters significantly simplify the developer experience and reduce the time required to integrate with various external systems.

Unlike other modules specific to each Abstract Account, Adapters are "global" in nature. This means that they are shared between multiple accounts. Due to this, **Adapter modules are not migratable**. This design choice is aimed at preventing supply-chain attacks that could potentially compromise the security of the Abstract ecosystem.

While individual Abstract Account owners can decide which Adapters and versions they wish to utilize, the overall control and maintenance of Adapters are handled at a platform level. This approach ensures that Adapters remain reliable, secure, and consistent across all Accounts.

```admonish example
The `abstract:dex` module allows Accounts to access standard functions on dexes with the same interface, regardless of whether they're local to the chain or across IBC.
```

```mermaid
flowchart LR
    subgraph Accounts
        direction BT
        subgraph Acc1["Account 1"]
            App1["abstract:etf"]
        end
        subgraph Acc2["Account 2"]
            App2["abstract:etf"]
        end
    end

    subgraph Adapters
        Acc1 --> Adapter1{{"abstract:dex"}}
        Acc2 --> Adapter1
    end

    Adapter1 --> dex1([Osmosis])
    Adapter1 --> dex2([Wyndex])
    Adapter1 --> dex3([Astroport])
```

<figcaption align = "center"><b>Two Accounts with the <code>abstract:etf</code> app module installed, using the <code>abstract:dex</code> adapter to interact with multiple dexes</b></figcaption>

## Installing and Uninstalling Modules

The following are sequence diagrams of the process of installing and uninstalling a module on an Abstract Account. As
you can see, the process happens via the Manager, and it can be done by the Account owner through
the [web-app](https://app.abstract.money/).

```mermaid
sequenceDiagram
    autonumber
    actor U as Owner
    participant M as Manager
    participant MF as Module Factory
    participant VC as Version Control
    participant P as Proxy
    U ->> M: InstallModule
    M ->> MF: InstallModule
    MF -->>+ VC: Query Module Details
    alt adapter
        VC -->>+ MF: Return address
    else app / standalone
        VC -->>- MF: Return code_id
        MF -> MF: Instantiate module
    end
    MF ->> M: Register module address
    M ->> P: Update module allowlist
```

<figcaption align = "center"><b>Installing a Module</b></figcaption>

At this point you should be able to understand the message flow depicted above. Just to be sure, we'll briefly outline the process's steps.

Installing a module starts by the Owner of the Account requesting the installation of the module on the Account. This request is sent to the Manager contract of the Account. The request contains the module ID(s) and possible instantiate messages for any App/Standalone modules that should be installed (aka instantiated).

The Manager contract verifies the request and forwards it to the Module Factory. The Module Factory then queries the Version Control (the on-chain module registry) for the module details. These module details contain both the version of the module as well as its type-specific data. This type-specific data is depicted by the two alternatives (alt) of the returned query.

Either the query returns an Adapter's address (which is already instantiated) or it returns an App/Standalone code-id. This code-id is then used by the Module Factory to instantiate an instance of that module.

After instantiating the modules the Manager registers the modules internally and updates the whitelist on the Account's Proxy contract. This whitelisting provides the modules with the ability to proxy message execution through the Account.

```mermaid
sequenceDiagram
    autonumber
    actor U as Owner
    participant M as Manager
    participant P as Proxy
    U ->> M: UninstallModule
    M -> M: Deregister module
    M ->> P: Update Proxy Whitelist
```

<figcaption align = "center"><b>Uninstalling a Module</b></figcaption>

Uninstalling a Module follows a similar execution flow as shown above.

```admonish info
In both flows we omitted the dependency-check logic, which will be discussed in more detail [later](../4_get_started/8_dependencies.md).
```

<!-- ## How to Use Modules in Your Project

Leveraging modules in your project can significantly simplify the development process, allowing you to deploy projects
faster and more cost-effectively than building traditionally. By using pre-built, tested, and community-reviewed
modules, you can focus on your project's unique features while reducing development time and minimizing potential bugs.

Here's an example of how modules can be leveraged for a decentralized finance (DeFi) project:

Imagine you want to create a DeFi application with the following features:

- A fungible token for your platform
- Staking and delegation functionality
- A governance system for community-driven decision-making
- Integration with an oracle for fetching off-chain data

Instead of building each feature from scratch, you can leverage Abstract's off-the-shelf modules to implement these
functionalities with ease. This not only saves time and resources but also ensures that your project benefits from the
best practices established by the Abstract community.

**Step 1**

Choose the module of your choice on the Abstract SDK. You can see the available modules on
our <a href="https://github.com/AbstractSDK/abstract/tree/main/modules" target="_blank">repository</a>.

**Step 2**

Import the chosen modules into your project and configure them according to your requirements. This can
include setting custom parameters, such as token supply, staking rewards, or voting thresholds.

**Step 3**

Integrate the modules with your existing codebase, ensuring they work seamlessly with your project's unique
features. This can involve calling module functions, implementing hooks, or extending your data structures.

**Step 4**

Test your dApp thoroughly to ensure the modules function as intended and do not introduce any unexpected
behavior.

By leveraging Abstract's modules in this way, you can rapidly build and deploy your DeFi project while benefiting from
the robustness and flexibility of the Abstract ecosystem. -->

## Example Execution Flows

The following are sequence diagrams of the process of executing a function on a module of an Abstract Account. We show three examples of executing a module: Owner Execution, Adapter Execution, and Dependency Execution.

Let's explore each of them.

### Owner Execution

To execute a (permissioned) message on a specific module, the Owner can call the module directly. The Module knows who the Owner of the Account is.

```mermaid
sequenceDiagram
    autonumber
    actor U as Owner
    participant Md as Module ("addr123")
    U ->> Md: Execute
```

### Adapter Execution

In the following example, the `abstract:dex` adapter is installed on an Account, and the Owner requests a swap on a dex. By providing the proxy address of the Account in the call, the adapter can assert that the caller is the Owner of the Account.

```mermaid
sequenceDiagram
    autonumber
    actor U as Owner
    participant D as abstract:dex
    participant VC as Version Control
    participant A as ANS
    participant P as Proxy
    participant T as Dex Pool
    Note right of U: Dex::Swap {proxy: "juno1xd..."}
    U ->> D: Call module
    D -->+ VC: Load proxy address for Account
    VC -->- D: Address
    D -->>+ A: Resolve asset names
    A -->> D: Asset infos
    D --> A: Resolve dex pool
    A -->>- D: Pool metadata
    D --> D: Build swap msg for target dex
    D ->> P: Forward execution
    Note over VC, A: DexMsg
    P ->> T: Execute
    Note right of P: DexMsg
```

### Dependency Execution

In this example, we use the [Equilibrium App](../7_use_cases/1_equilibrium.md)'s `Rebalance` function as an example. Modules
with dependencies (`equilibrium:balancer` is dependent on `abstract:etf` and `abstract:dex`) have their addresses
dynamically resolved when called.

```mermaid
sequenceDiagram
    autonumber
    actor U as User
    participant B as equilibrium:balancer
    participant P as Proxy
    participant M as Manager
    participant D as abstract:dex
    participant T as Target Dex
    U ->> B: Rebalance
    B -->>+ P: Query Allocations
    P -->>- B: Allocations
    B --> B: Calculate rebalancing requirements
    B -->>+ M: Query abstract:dex address
    M -->>- B: Address
    B ->> D: Call SwapRouter on dex
    D --> D: Build swap msg for target dex
    D --> D: Load proxy address
    D ->> P: Forward execution
    Note over M: DexMsg
    P ->> T: Execute
    Note over D, M: DexMsg
```
