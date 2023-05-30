# Uninstall Module


```mermaid
sequenceDiagram
  autonumber
  actor U as User
  participant M as Manager
  participant P as Proxy

  U->>M: UninstallModule
  M-->M: Check dependencies
  M->M: Deregister module
  M->>P: Update module allowlist
```
