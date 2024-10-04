# Module Factory

The Module Factory is a contract that allows Account owners to install and manage Abstract Modules for their Account. You can install modules by interacting with the Account directly, i.e. via CLI, or by using the [Account Console](4_account_console.md).

To recap from [that chapter](../3_framework/6_module_types.md), there are three types of modules: App, Adapter, and Standalone.

## Flow Diagrams

### Install Module

When a developer requests the installation of a module, the following internal process is initiated:

```mermaid
sequenceDiagram
    autonumber
    actor U as Owner
    participant A as Account
    participant MF as Module Factory
    participant REG as Registry

    U ->> A: InstallModule
    A ->> MF: InstallModule
    MF -->>+ REG: Query reference
    alt adapter
        REG -->>+ MF: Return address
    else app / standalone
        REG -->>- MF: Return code_id
        MF -> MF: Instantiate module
    end
    MF ->> A: Register module address

    A ->> A: Update module allowlist
```

<figcaption align = "center"><b>Installation of a module</b></figcaption>

## Execute on Module

Once the module is installed, there are essentially three ways to interact with it depending on the type of module:

### Owner Execution

The owner of an Account can always execute on the module directly, even if the module is installed on a sub-account.

```mermaid
sequenceDiagram
    autonumber
    actor U as Owner
    participant Md as Module ("addr123")
    Note right of U: ModuleMsg

    U ->> Md: Execute
```

<figcaption align = "center"><b>Module Execution</b></figcaption>

### Adapter Execution

In the following example, the `abstract:dex` module is installed on an Account and the user requests a swap on a dex. This swap will use the funds held in the Account to execute the swap on the target dex.

```mermaid
sequenceDiagram
    autonumber
    actor U as Owner
    participant D as abstract:dex
    participant A as ANS
    participant A as Account
    participant T as Dex Pool
    Note right of U: Dex::Swap {proxy: "juno1xd..."}
    U ->> D: Call module
    D -->>+ A: Resolve asset names
    A -->> D: Asset infos
    D --> A: Resolve dex pool
    A -->>- D: Pool metadata
    D --> D: Build swap msg for target dex
    D ->> A: Forward execution
    Note over D, A: DexMsg
    A ->> T: Execute
    Note right of A: DexMsg
```

<figcaption align = "center"><b>Adapter Execution</b></figcaption>

### App Execution w/ Dependencies

In this example, we use [Equilibrium](../7_use_cases/equilibrium.md)'s `Rebalance` permissionless function as an example. Modules with
dependencies (`equilibrium:balancer` is dependent on `abstract:etf` and `abstract:dex`) have their addresses dynamically
resolved when called.

```mermaid
sequenceDiagram
    autonumber
    actor U as User
    participant B as equilibrium:balancer
    participant A as Account
    participant D as abstract:dex
    participant T as Target Dex

    U ->> B: Rebalance
    B -->>+ A: Query Allocations (Oracle)
    A -->>- B: Allocations
    B --> B: Calculate rebalancing requirements

    B -->>+ A: Query abstract:dex address
    A -->>- B: Address

    B ->> D: Call SwapRouter on dex
    D --> D: Build swap msg for target dex
    D --> D: Load account address

    D ->> A: Forward execution
    Note over A: DexMsg
    A ->> T: Execute
    Note over D, A: DexMsg
```

<figcaption align = "center"><b>Dependent Execution</b></figcaption>
