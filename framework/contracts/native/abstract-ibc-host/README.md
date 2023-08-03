# Abstract IBC host

The Abstract IBC host is a contract designed to be deployed on an Abstract-deployed chain. It enables any Account on a remote chain to claim and perform cross-chain actions on an account on the local chain.

## Supported actions

### Register

Register an Account by creating a local account. Incoming requests and funds will be routed to this account.

### Dispatch

Proxy a set of execute messages to the Account's proxy on the host chain.

## IBC

Channel creation is permissionless but must be done between the host and an approved client address. Channels that are created between this host and the client will be mapped to a chain-name. This chain-name is used to identify the chain when performing cross-chain actions.

In case of changes to the network's chain-id the chain name will remain identical. This enables easy migration by clearing the 