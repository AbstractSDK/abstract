# Update Settings 
## Enable IBC
Enabling the IBC functionality of your account will register the IBC client to your account, enabling your modules to execute cross-chain commands.
```mermaid
sequenceDiagram
  autonumber
  actor U as User
  participant M as Manager
  participant VC as Version Control
  participant P as Proxy

  U->>M: UpdateSettings
  Note right of U: ibc_enabled
    M-->>+VC: Query IBC Client reference
    VC-->>-M: Return IBC Client address
    M->>M: Register IBC Client
    M->>P: Add IBC client to allowlist

```

> For disabling IBC, see [Uninstall Module](uninstall_module.md)