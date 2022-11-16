# Abstract IBC architecture

The Abstract IBC architecture aims to provide developers with a set of Abstract-SDK supported actions to simplify IBC usage. 

# Message flow
IBC actions are instantiated in a custom contract (with proxy execute permissions) or an installed API/add-on. They result in a call to the OS's proxy contract on the `ExecuteMsg::IbcAction { msgs: Vec<IbcClientMsg> }` endpoint. 

These `IbcClientMsg` messages are then called on the OS's client contract. Note that the client contract must be enabled on the OS's manager. This ensures that the user/developer is aware of enabling IBC on their OS. 

> By calling the client through the OS's proxy we can ensure the calling contract has sufficient permission to perform action on the local and remote OS.
> The IBC functionality can be enabled on the OS by calling `EnableIbc` on the manager contract. 
The client contract will check the caller's identity and packet destination. It will then construct the packet and send it over IBC. 

> The channel over which these packets are relayed is maintained by Abstract. Nonetheless we advise users to also relay the channel using their own relayer.  

# IBC Client
The IBC client contract is a single contract deployed to the client chain (the chain on which the developer aims to deploy his application). The client contract can only be called by an OS proxy. By providing an [`abstract_os::ibc_client:ExecuteMsg::SendPacket`] message the Client contract will resolve the target chain and related IBC channel to send the packet over. An action and optional callback data is also included in the message. The optional callback data is used to perform a callback by the client after they received an `Ack::Success` for the specific packet. 

The callback receiver should implement the following in their contract execute message:  
```rust
pub enum ExecuteMsg{
    IbcCallback(IbcResponseMsg),
}
```
Where the `IbcResponseMsg` is defined as:  
```rust
pub struct IbcResponseMsg {
    pub id: String,
    pub msg: StdAck,
}
```
The response ID can then be matched in the receiving contract to identify the action that has finished, along with parsing the Binary response for successful actions.
This functionality is already provided by the add-on and api contract implementations. 

> Abstract's packages provide an easy entrypoint to this functionality. 

The IBC client receives the mentioned data and constructs an IBC packet that contains the most important data for later processing. This data is contained in the [`abstract_os::ibc_host::PacketMsg`] struct. 


# IBC Host
The IBC host is a packaged contract that can be used to create a contract that interfaces with the chain-specific logic of the chain on which it is deployed. By providing this as as base-implementation we believe adding new chains and their functionality should be trivial. 

It accepts [`abstract_os::ibc_host::PacketMsg`] messages over IBC. A custom `packet_handler` function is expected to be implemented. By providing the type-information about what specific data is expected, any `HostAction::App(Binary)` calls will be deserialized to the expected type. For example, the `osmosis-host` contract is defined as `pub type OsmoHost<'a> = Host<'a, abstract_os::dex::RequestMsg>`. Therefore any packets sent to this host with an `HostAction::App(Binary)` variant will be deserialized to a `abstract_os::dex::RequestMsg` which can then be parsed into the local stargaze messages that are required to perform the specified DEX action on the associated proxy contract. 

> If a channel to the host is closed, it is added to a `closed_channels` list. Any proxy contracts instantiated under this channel are now locked. The `SendAllBack` action is allowed to be called freely, enabling users to get any funds that were left in the proxy back. 

# ICS20 transfers 
In order to easily perform token transfers between chains, a channel-id store was added to the AnsHost contract. This allows any contract to locally resolve which channel to use in order to transfer some token to a different chain. The store also allows for more channel entries in the future. 
