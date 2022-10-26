# Abstract IBC host

The Abstract IBC host is a contract designed to be deployed on a remote App chain. It enables any OS to perform cross-chain actions on a proxy account owned by the OS. This contract is offered as a package and should be used as a starting point for implementing stateless execution logic on a remote chain. 

## Supported actions

### Register
Register an OS by creating a CW1 proxy for the OS and assigning a map entry to it. This way incoming IBC requests can be sent to the correct proxy. 

### Dispatch
Proxy a set of execute messages to the OS's proxy on the host chain. 

### App
Provide the app-specific execute messages that resolve into a `Vec<CosmosMsg>` that's then called on the proxy contract.
This endpoint provides client chains with an interface to the application logic available on the app chain. 

