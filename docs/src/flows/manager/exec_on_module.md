# Execute On Module

## Non-dependent Execution
```mermaid
sequenceDiagram
  autonumber
  actor U as User
  participant M as Manager
  participant Md as Module

  U->>M: ExecOnModule
  Note right of U: ModuleMsg
  
  M-->>M: Load module address
  M->>Md: Execute
  Note right of M: ModuleMsg
```

## Adapter Execution
In the following example, the `abstract:dex` module is installed on an Account, and the user requests a swap on a dex. 

```mermaid
sequenceDiagram
  autonumber
  actor U as User
  participant M as Manager
  participant D as abstract:dex
  participant VC as Version Control
  participant A as ANS
  participant P as Proxy
  participant T as Dex Pool

  U->>M: ExecOnModule
  Note right of U: Dex::Swap
  M-->M: Load module address
  M->>D: Call module
  Note right of M: Adapter Msg
  D-->+VC: Load proxy address for Account
  VC-->-D: Address

  D-->>+A: Resolve asset names
  A-->>D: Asset infos
  D-->A: Resolve dex pool
  A-->>-D: Pool metadata
  D-->D: Build swap msg for target dex
  
  D->>P: Forward execution
  Note over VC,A: DexMsg
  P->>T: Execute
  Note right of P: DexMsg

```

## Module-dependent Execution
In this example, we use [Equilibrium](../use_cases/equilibrium)'s `Rebalance` function as an example. Modules with dependencies (`equilibrium:balancer` is dependent on `abstract:etf` and `abstract:dex`) have their addresses dynamically resolved when called. 

```mermaid
sequenceDiagram
  autonumber
  actor U as User
  participant B as equilibrium:balancer
  participant P as Proxy
  participant M as Manager
  participant D as abstract:dex
  participant T as Target Dex

  U->>B: Rebalance
  B-->>+P: Query Allocations
  P-->>-B: Allocations
  B-->B: Calculate rebalancing requirements

  B-->>+M: Query abstract:dex address
  M-->>-B: Address

  B->>D: Call SwapRouter on dex
  D-->D: Build swap msg for target dex
  D-->D: Load proxy address
  
  D->>P: Forward execution
  Note over M: DexMsg
  P->>T: Execute
  Note over D,M: DexMsg

```

