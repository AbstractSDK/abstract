# Module Dependencies

In the Abstract SDK, modules have conditions that must be met before they can be registered or activated. These
conditions largely revolve around module dependencies and version requirements. When installing a module, the system will
check its dependencies and ensure that they are installed and meet the version requirements specified by the module.

Here's how the process of registering a module and checking module dependencies looks:

```mermaid
sequenceDiagram
    autonumber
    actor U as User
    participant M as Manager
    participant VC as Version Control
    participant P as Proxy

    U ->> M: RegisterModule
    M -->>+ VC: Assert Install Requirements
    VC -->> VC: Load Module Dependencies
    VC -->> M: Assert Dependency Requirements
    VC -->> VC: Check if Dependency is Installed
    VC -->> VC: Assert Version Requirements
    M -->>+ VC: Add Module as Dependent on its Dependencies
    M -->>+ P: Add Module
```
