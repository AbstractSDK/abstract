# Module Dependencies

In the Abstract SDK, modules have conditions that must be met before they can be installed. These
conditions largely revolve around module dependencies and version requirements. When installing a module, the system will
check its dependencies and ensure that they are installed and meet the version requirements specified by the module.

Here's how the process of installing a module and checking module dependencies looks:

```mermaid
sequenceDiagram
    autonumber
    actor U as Owner
    participant M as Manager
    participant F as Module Factory
    participant Mo as Module
    participant P as Proxy

    U ->> M: Install Module
    M -->> F: Install Module
    opt App instantiate 
    F -->> Mo: Instantiate Module
    end
    M -->> Mo: Query Module Dependencies
    M -->> M: Assert Dependency Requirements
    M -->>+ M: Add Module as Dependent on its Dependencies
    M -->>+ P: Allowlist Module
```
