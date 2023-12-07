# Abstract IBC

## Synopsis

This standard document specifies packet data structure, state machine handling logic, and encoding details for the transfer of messages and creation of Abstract accounts over an IBC channel between a client and a host on separate chains. The state machine logic presented allows for safe multi-chain account creation and execution.

### Motivation

Users of a set of chains connected over the IBC protocol might wish to interact with smart-contracts and dapps present on another chain than their origin, while not having to onboard the remote chain, create a new wallet or transfer the necessary funds to this other chain. This application-layer standard describes a protocol for interacting with a remote chain and creating Abstract Account on chains connected with IBC which preserves asset ownership, limits the impact of Byzantine faults, and requires no additional permissioning.

### Definitions

The Abstract IBC Account interface is described in the following guide and the specifications are roughly presented here

### Desired Properties

- Preservation of account and funds ownership
- All interactions that can be done by a local account should be possible for a remote account as well.

## Technical Specification

### Data Structures

Only one packet data type is added in this spec to be able to interact across IBC chains

```rust
pub struct PacketMsg {
    /// Chain of the client
    pub client_chain: String,
    /// Amount of retries to attempt if packet returns with StdAck::Error
    pub retries: u8,
    pub account_id: AccountId,
    /// Callback performed after receiving an StdAck::Result
    pub callback_info: Option<CallbackInfo>,
    /// execute the custom host function
    pub action: HostAction,
}
```

#### Execution

- client_chain specifies the chain from which the message originates. Once a channel is created between client and host, this channel will always be checked to match the registered configuration

- account_id specifies the account that is calling the action on the local chain.

- action specifies what the remote chain should execute upon receiving this packet

#### Acknowledgement

When the action is executed on the remote chain, it can either be successful or yield an error.

- retries specifies the number of attemps left to submit the packet. In case an error is yielded by the remote chain, the original packet will be sent back to the original chain and retried as long as retries > 0. Because IBC actions are asynchronous, some packets may need to wait other packet to go through before they can be executed. This parameter allows the packet action to fail multiple times before it's indeed sent across a channel

- call_back_info is an optional object that specifies any action that needs to be executed after the packet has been sucessfully executed and a positive (`StdAck::Result`) acknowledgement has been transfered back. 


#### Cross chain trace

Because accounts created across chains using the IAA protocol are controlled by an account located on a remote chain, the `account_id` parameter should specify which chain is calling an action. In order to follow which chains a message is called from, the IBC Abstract module leverages the `AccountId::trace` field. An account is wether `AccountTrace::Local` or `AccountTrace::Remote`. When a PacketMsg is sent across an IBC channel, the account id is transformed in the following manner : 
- If it was `AccountTrace::Local` before transfer, it turns into an `AccountTrace::Remote` account with one chain in the associated vector being the chain calling the `PacketMsg` (`PacketMsg::client_chain`)
- If it was `AccountTrace::Remote` before transfer, it stays remote and the `client_chain` field is added to the associated vector.

This allows full traceability of the account creations and calls.
