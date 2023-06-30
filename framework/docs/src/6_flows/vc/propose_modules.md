# Propose Modules
Developers that wish to publish modules to the Abstract platform need to call `ProposeModules` on the [Version Control](../../platform/version_control.md) contract. The modules will subsequently be reviewed by the Abstract platform for registration.

> For documentation on how to register modules, see [Module Deployment](../../4_get_started/module_deployment.md)

Modules cannot be registered without their namespaces being claimed by an Account. This is to prevent malicious actors from registering modules under trusted namespaces.

Below details the assertion process.

```mermaid
sequenceDiagram
  autonumber
  actor U as User

  participant VC as Version Control
  participant Man as Manager of Namespace
  participant M as Adapter Instance

  U->>VC: ProposeModules

  loop

    VC-->VC: Load Account ID for namespace
    VC-->VC: Load Account Manager address
    VC-->>+Man: Query Account owner
    Man-->>-VC: Address

    opt adapter
        VC-->>M: Assert no sudo admin
        activate M
        VC-->>M: Assert same cw2 data
        deactivate M
    end

    end
    VC->>VC: Save modules

```