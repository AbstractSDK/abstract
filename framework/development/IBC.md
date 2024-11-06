# Abstract IBC architecture

The Abstract IBC architecture aims to provide developers with a set of Abstract-SDK supported actions to simplify IBC usage. 

# Message flow
IBC actions are instantiated in a custom contract (with Account execute permissions) or an installed adapter/app. They result
in a call to the Account contract on the `ExecuteMsg::IbcAction { msgs: Vec<IbcClientMsg> }` endpoint.

These `IbcClientMsg` messages are then called on the Account's client contract. Note that the client contract must be enabled
on the Account. This ensures that the user/developer is aware of enabling IBC on their Account.

> By calling the client through the Account we can ensure the calling contract has sufficient permission to perform action on the local and remote Account.
> The IBC functionality can be enabled on the Account by installing the `IBC_CLIENT` adapter on the account.
> The client contract will check the caller's identity and packet destination. It will then construct the packet and send it over IBC.
> The channel over which these packets are relayed is maintained by Polytone and Abstract. Nonetheless we advise users to also relay the channel using their own relayer. (The relaying strategy still has to be determined)

# IBC Client
The IBC client contract is a single contract deployed to the client chain (the chain on which the developer aims to
deploy his application). The client contract can only be called by an Account. By providing
an [`abstract_sdk::std::ibc_client:ExecuteMsg::RemoteAction`] message, the Client contract will resolve the target
chain and related Polytone contracts to send the message through.

```rust
RemoteAction {
        // host chain to be executed on
        // Example: "osmosis"
        host_chain: String,
        // execute the custom host function
        action: HostAction,
        // optional callback info
        callback_info: Option<CallbackInfo>,
    }    
```

An action and optional callback data is also included in the message : 
- The action indicates the remote action that the user wants to execute on the remote chain.
- The optional callback data is used to perform a callback by the ibc-client after the action on the remote chain has resolved. 

    To be more precise, whenever an ibc action is executed on the remote chain, the result of the action (even if it failed), is sent back to the ibc-client on the calling chain. When the ibc-client receives that result and if the callback_info is set, it will send this result to the registered `callback_info.receiver`, by executing the following msg on the receiver execute endpoint

    ```rust
    pub enum ExecuteMsg{
        IbcCallback(IbcResponseMsg),
    }
    ```
    Where the `IbcResponseMsg` is defined as:  
    ```rust
    pub struct IbcResponseMsg {
        pub id: String,
        pub result: Callback, // This is the Polytone Callback Enum
    }
    ```

    If your contract is supposed to receive abstract-ibc callbacks, you simply need to implement the `IbcCallback` variant on your contract  `ExecuteMsg`.

    The response `id` field can then be matched in the receiving contract to identify the action that has finished, along with
    matching the `result` field for successful `Execute` actions.
    This functionality is already provided by the app and adapter contract implementations.

    > Abstract's packages provide an easy entrypoint to this functionality.


# IBC Host

The IBC host is a contract that is included in Abstract's deployments as well and that is responsible for authenticating the messages it receives from its IBC-Client counterparts. It is very general and can handle executing arbitrary messages on remote Abstract accounts.

It accepts [`abstract_sdk::std::ibc_host::HostAction`] messages over IBC. This HostAction can either be : 
- Registering an account on the local chain from a remote chain
- Executing an action on a local account from a remote chain. In case this action is called, when no local account attached to the calling remote account was previously created, an account on the local chain will be created automatically on behalf of the calling remote account.
- Send all native funds back to the calling remote account 


# Connection between IBC Client and IBC Host 

The IBC Client and IBC Host don't manage the IBC connection between themselves directly. Today, a [Polytone](https://github.com/DA0-DA0/polytone/) Channel is responsible for making that connection. Using an existing IBC solution that leverages a single channel for all interchain accounts registered with it allows us to group the relaying efforts needed and simplify maintenance for our IBC channels.

## IBC channel creation
    Creating a connection between the client and the host is triggered by the ibc-client when a new chain is registered on the contract. It sends a message to the remote chain and retrieves the address that will be dispatching messages to the remote host. 
    Then, this address needs to be registered with the remote host. Only this address will be able to execute `HostAction` messages on the remote host. 

    In case a channel closes or whatever other failing case of the IBC solution, it's possible to un-register the associated chain with the ibc-host and ibc-client.
    When the IBC solution is restored, the maintainer can easily re-register the chain without any data/connection/funds lost in the maneuver. 

    This allows us additionally to change the IBC solution in the future if needed, without too much complexity.


# ICS20 transfers 

In order to easily perform token transfers between chains, a channel-id store was added to the AnsHost contract. This allows any contract to locally resolve which channel to use in order to transfer some token to a different chain. The store also allows for more channel entries in the future. 

Today, as long as the callbacks and hooks are still not implemented and available on chains, there is no simple way to send funds to a remote chain while executing an action. In the future, we will allow this on all Abstract Accounts. Investing/Removing your funds from remote protocols will become simple as ever. [Learn more about this here](https://medium.com/the-interchain-foundation/introducing-the-callbacks-middleware-compose-smart-contracts-and-modules-with-ibc-6f3fb527e44a).
