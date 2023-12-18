# Version Control

The Version Control contract acts as the registry for all modules and accounts within the Abstract platform. Abstract Accounts can use it to claim namespaces and register their modules. The Version Control contract allows modules to be queried by its namespace, name, and version, returning its reference which may be a code id or address.

## Propose Modules

Developers that wish to publish modules to the Abstract platform need to call `ProposeModules` on the Version Control contract. The modules will subsequently be reviewed by the Abstract platform for registration.

```admonish info
For documentation on how to register modules, see [Module Deployment](../4_get_started/7_module_deployment.md)
```

Modules cannot be registered without their namespaces being claimed by an Account. This is to prevent malicious actors from registering modules under trusted namespaces.

Below details the assertion process.

```mermaid
sequenceDiagram
    autonumber
    actor U as Owner

    participant VC as Version Control
    participant Man as Manager of Namespace
    participant M as Adapter Instance

    U ->> VC: ProposeModules

    loop

        VC --> VC: Load Account ID for namespace
        VC --> VC: Load Account Manager address
        VC -->>+ Man: Query Account owner
        Man -->>- VC: Address

        opt adapter
            VC -->> M: Assert no sudo admin
            activate M
            VC -->> M: Assert same cw2 data
            deactivate M
        end

    end
    VC ->> VC: Save modules

```

```admonish warning
For mainnet deployment proposed modules are reviewed by the Abstract team. To get them approved, reach out to us on [Discord](https://discord.com/invite/uch3Tq3aym). For testnet deployment there is no review process.
```
