# Abstract IBC host

The Abstract IBC host is a contract designed to be deployed on an Abstract-deployed chain. It enables any Account on a remote chain to claim and perform cross-chain actions on an account on the local chain.

## Supported actions

### Register

Register an Account by creating a local account. Incoming requests and funds will be routed to this account.

### Dispatch

Proxy a set of execute messages to the Account's proxy on the host chain.

## IBC

In order to connect this host contract to the client contract on a remote chain, one need to use an IBC protocol that can relay packets between chains. In our case, we use Polytone by DA0-DA0, which provides great flexibility and built in security (by not mixing up chains), as well as common channels with all other protocols that build cross chain apps. This allows for easier maintainability and collaboration. 

This host contract need to whitelist Polytone's execution proxy for each chain it communicates with. IBC Channels are therefore identified with the chain name, which allows for easy migration, even when the chain id changes !