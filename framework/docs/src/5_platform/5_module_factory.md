# Module Factory

The Module Factory is a contract that allows you to install and manage Abstract Modules via the Account Manager. You can
install modules by interacting with the Account Manager directly, i.e. via CLI, or by using
the [Account Console](4_account_console.md).

To recap from [that chapter](../3_framework/6_module_types.md), there are three types of modules: App, Adapter, and Standalone.

# Flow Diagrams

## Install Module

When a developer requests the installation of a module, the following internal process is initiated:

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
    MF -->>+ VC: Query reference
    alt adapter
        VC -->>+ MF: Return address
    else app / standalone
        VC -->>- MF: Return code_id
        MF -> MF: Instantiate module
    end
    MF ->> M: Register module address

    M ->> P: Update module allowlist
```

<figcaption align = "center"><b>Installation of a module</b></figcaption>

## Execute on Module

Once the module is installed, there are essentially three ways to interact with it depending on the type of module:

### Owner Execution

```mermaid
sequenceDiagram
    autonumber
    actor U as Owner
    participant M as Manager
    participant Md as Module

    U ->> M: ExecOnModule
    Note right of U: ModuleMsg

    M -->> M: Load module address
    M ->> Md: Execute
    Note right of M: ModuleMsg
```

<figcaption align = "center"><b>Non-dependent Execution</b></figcaption>

### Adapter Execution

In the following example, the `abstract:dex` module is installed on an Account, and the user requests a swap on a dex.

```mermaid
sequenceDiagram
    autonumber
    actor U as Owner
    participant M as Manager
    participant D as abstract:dex
    participant VC as Version Control
    participant A as ANS
    participant P as Proxy
    participant T as Dex Pool

    U ->> M: ExecOnModule
    Note right of U: Dex::Swap
    M --> M: Load module address
    M ->> D: Call module
    Note right of M: Adapter Msg
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

<figcaption align = "center"><b>Adapter Execution</b></figcaption>

### User Execution

In this example, we use [Equilibrium](../../use_cases/equilibrium.md)'s `Rebalance` permissionless function as an example. Modules with
dependencies (`equilibrium:balancer` is dependent on `abstract:etf` and `abstract:dex`) have their addresses dynamically
resolved when called.

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

<figcaption align = "center"><b>Module-dependent Execution</b></figcaption>
