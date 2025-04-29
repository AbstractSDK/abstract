# Using IBC to interact with EVM chains

## Introduction

[Abstract Accounts](../3_framework/3_architecture.md) have the capability to control Accounts on any <a href="https://docs.union.build/protocol/connections/overview/" target="_blank" >EVM chain supported by Union</a>. This allows any Abstract developer to seemlessly integrate with any logic or Protocol on any EVM chain. This allows users to bridge liquidity, swap and interact with their favorite EVM protocols without leaving your home chain. In this page, we will discuss the design of those interaction a well as introduce how to integrate this functionality inside your contracts, your tests and even your frontend. 

## Design and high-level considerations

### Transfering funds

Union provides the <a href="https://docs.union.build/protocol/channels/ucs01-relay/" target="_blank">ucs01</a> protocol to send funds to EVM chain. Go down below to the section devoted to [Funds transfer implmentaiton](#transfering-funds-within-smart-contracts) to learn how to use this capability within smart contracts.

### Sending messages across

Union also allows connecting contracts through IBC channels. Abstract has created a way for any user to create a remote account on EVM chains. This solution is a port of <a href="https://github.com/DA0-DA0/polytone/" target="_blank">`Polytone`</a> adapter to Union implementation of IBC in Solidity. Just like for Abstract Accounts on Cosmos chains, each user actually has an account onevery EVM chain through this implementation. In the follwing diagram, we present the different contracts used in this implementation:

```mermaid
flowchart LR
    subgraph CosmWasm Chain
        PV[Polytone Voice]
        PP[Polytone Proxy]
        PP -.execute.-> SC3[Smart Contract]
    end

    subgraph Ethereum Chain
        A6[EVM Proxy]
        EV[EVM Voice]
        EV -.-> A6
        A6 -.delegate_call/call.-> SC[Smart Contract]
    end

    subgraph CosmWasm Chain
    direction TB
    A3 -.-> PN[Polytone Note]
    A3 -.-> EN[EVM Note]

    end

    A3["EOA / Smart Contract"]
    PN -.-> PV
    PV -.-> PP
    EN -.-> EV
```

Go down below to the section devoted to [Sending messages across](#sending-messages-across-within-smart-contracts) to learn how to use this capability within smart contracts.

## Smart Contract integration

In this section, we present how to send messages and funds to EVM chains using IBC connections. The following steps are specific to EVM and differ from interactions with [Cosmos Interchain Abstract Accounts](./account-ibc.md).

### Transfering funds within smart-contracts

To transfer funds to EVM chains, you need to use the `abstract:ica-client` module. This contract is used to retrieve the necessary messages that will need to be sent from the account. Here are the steps necessary to be able to call this contract.

1. Retrieve the ICA-client address : TODO
2. Call the module to query the messages that you will need to send : 
   ```rust

    

   ```

### Sending messages across within smart-contracts

## Frontend integration

## Integrating with Interchain Abstract Accounts

### Enabling IBC on the Account

Users must install the `ibc-client` contract on their account to enable IBC. To do so they can call the `ExecuteMsg::InstallModules` endpoint with the `abstract:ibc-client` module ID.

```rust
pub enum ExecuteMsg{
    InstallModules {
        // Module information (module_id and version)
        modules: Vec<ModuleInstallConfig>,
    },
    //...
}
```

### (Optional) Create an account on the remote chain

After the initialization step, the account is ready to send messages across IBC. However, if you wish, you can customize the remote account metadata before sending any messages. The following message is executed on the [`account`](https://docs.rs/abstract-std/latest/abstract_std/account/index.html) contract:
<pre>
<code class="language-rust">pub enum <a href="https://docs.rs/abstract-std/latest/abstract_std/account/enum.ExecuteMsg.html" target="blank">AccountExecuteMsg</a> {
    ExecuteOnModule {
        module_id: "abstract:ibc-client",
        exec_msg: <a href="https://docs.rs/abstract-std/latest/abstract_std/ibc_client/enum.ExecuteMsg.html" target="blank">IbcClientExecuteMsg</a> {
            Register {
                host_chain: "destination-chain",
                // Customizable parameters
                base_asset: None,
                namespace: None,
                install_modules: vec![],
            },
            ...,
        }
    }
    ...,
}
</code>
</pre>

```admonish info
Remember that this step is optional as accounts are created automatically when sending the first message across IBC.
```

### Account ID structure

The remote Interchain Abstract Account will have the same account sequence but will have a different trace. Let's take an example. A account on `Neutron` with account sequence `42` wants to create accounts on `Osmosis` and `Stargaze`.

- Their account ID on `Neutron` is `local-42`.
- Their account ID on `Osmosis` is `neutron-42`.
- Their account ID on `Stargaze` is `neutron-42` as well!

Remote accounts can create other remote accounts, and their traces will be chained. For instance, the `neutron-42` account on `Osmosis` can create an account on `Stargaze` which will have the ID `osmosis>neutron-42`.
This gives the ability to trace ICAAs back to their origin chain.


### Sending messages on remote accounts

With or without a pre-existing remote Account, Abstract Accounts are able to send messages on remote Accounts. The `account_msgs` will be executed in order on the remote account.

<pre>
<code class="language-rust">pub enum <a href="https://docs.rs/abstract-std/latest/abstract_std/account/enum.ExecuteMsg.html" target="blank">AccountExecuteMsg</a> {
    ExecuteOnModule {
        module_id: "abstract:ibc-client",
        exec_msg: <a href="https://docs.rs/abstract-std/latest/abstract_std/ibc_client/enum.ExecuteMsg.html" target="blank">IbcClientExecuteMsg</a> {
            RemoteAction{
                host_chain: "destination-chain",
                action: <a href="https://docs.rs/abstract-std/latest/abstract_std/ibc_host/enum.HostAction.html" target="blank">HostAction</a>{
                    Dispatch{
                        account_msgs: Vec<<a href="https://docs.rs/abstract-std/latest/abstract_std/account/enum.ExecuteMsg.html" target="blank">AccountExecuteMsg</a> { ... }>
                    },
                    ...,
                }
            },
            ...,
        }
    }
    ...,
}
</code>
</pre>

Note that the two instances of the `AccountExecuteMsg` enum are the exact same type. This allows you to send multi-hop IBC messages. However, multi-hop transactions (of these kind) are not really something you would use often, unless you're using another chain as a routing chain.

## Specification of Interchain Abstract Accounts

The following specification specifies packet data structure, state machine handling logic, and encoding details for the transfer of messages and creation of Abstract accounts over an IBC
channel between a client and a host on separate chains. The state machine logic presented allows for safe multi-chain account creation and execution.

### Motivation

Users of a set of chains connected over the IBC protocol might wish to interact with smart-contracts and dapps present on another chain than their origin, while not having to onboard the remote chain, create a new wallet or transfer the necessary funds to this other chain.
This application-layer standard describes a protocol for interacting with a remote chain and creating Abstract Account on chains connected with IBC which preserves asset ownership,
limits the impact of Byzantine faults, and requires no additional permissioning.

### Definitions

The Abstract IBC Account interface is described in the following guide and the specifications are roughly presented here

### Desired Properties

- Preservation of account and funds ownership
- All interactions that can be done by a local account should be possible for a remote account as well.

### Technical Specification

#### General mechanism

Abstract IBC capabilities are allowed by the `ibc-client`<->`ibc-host` pair. The `ibc-client` is responsible for authenticating the sender and sending packets across IBC to the `ibc-host`. The `ibc-host` is responsible for receiving packets and routing the packet to the corresponding contract on the remote chain. Under the hood,the `client`-`host` connection is handled by a <a href="https://github.com/DA0-DA0/polytone" target="blank">Polytone</a> channel. This allows Abstract to be interoperable with other protocols, more resilient to IBC constraints. However, Abstract is not bound to Polytone and any other IBC relaying protocol could be used for relaying packets. Here is a simple schematic that explains the different components of the Abstract IBC infrastructure.

```mermaid
flowchart LR

    subgraph Osmosis[Osmosis Chain]
        Osmosis-Abstract-Account -.-> Osmosis-Client
        Osmosis-Client --> Osmosis-Note
        subgraph Osmosis-Polytone[Polytone]
            Osmosis-Note
        end
    end


    subgraph Neutron[Neutron Chain]
        subgraph Neutron-Polytone[Polytone]
            Neutron-Voice -.-> Neutron-Proxy
        end
        Neutron-Proxy --> Neutron-Host
        Neutron-Host -.-> Neutron-Abstract-Account
    end

    Osmosis-Note ==IBC==> Neutron-Voice
```

You see that an Abstract Interchain connection is uni-directional. You need 2 connections to be able to interact bi-directionnally with Abstract. Up until today however, only a local account can act on a distant account and not the other way around. Here is an examples using `AccountId` between `neutron` and `osmosis`:

- `local-42` on `neutron` **CAN** control `neutron-42` on `osmosis` via IBC
- `neutron-42` on `osmosis` **CAN'T** control `local-42` on `neutron`

##### Account creation

Interchain Abstract Accounts are traditional Abstract Accounts controlled by the ibc-host. The ibc-host is the admin of the account and routes any packet sent by a remote account on the corresponding local account. When creating an abstract account, it is simply registered by the `ibc-host` using the [`account-factory`](../5_platform/3_account_creation.md) just like any other account.

When an action is triggered by a remote account, the `ibc-host` does the following verifications:

- If an local account already exists on-chain for the remote account, it just dispatches the message to the account.
- If no account exists, it creates one with default metadata and THEN dispatches the messages to this new account.

The Account creation process is therefore not mandatory when interacting with Interchain Abstract Accounts. This is why when you create an Abstract Account, you automatically have an account on every connected chains!

#### Data Structures

Interchain Abstract Account communication is done via a single message structure:

```rust
pub enum IbcHostExecuteMsg{
{{#include ../../../packages/abstract-std/src/native/ibc/ibc_host.rs:ibc-host-execute}}
    ...,
}
```

- `account-id` is the `id` of the local account calling the action.
- `account_address` is the address of the local account calling the action.
- `action` is the action that should be executed by the `ibc-host` on the remote account:

```rust
{{#include ../../../packages/abstract-std/src/native/ibc/ibc_host.rs:ibc-host-action}}
```

#### Acknowledgement and Callback

IBC works with 4 steps:

  1. Sending a packet (Source chain)
  2. Receiving a packet (Destination chain)
  3. Sending an acknowledgement (Destination chain)
  4. Receiving an acknowledgement (Source chain)

We have already covered the 2 first steps with the sections above. We cover the 2 lasts steps in this section.

Step 3 (sending an ack), is handled by Polytone. They catch any error that could happen during contract execution and send back an acknowledgement reflecting the state of the contract execution on the remote chain. This is handled through the <a href="https://docs.rs/polytone/latest/polytone/callbacks/enum.Callback.html" target="blank">Callback struct</a>.

For Step 4, Polytone allows for sending a message to the initial sender of the IBC interaction after the packet was successfully received in the remote chain. Abstract **DOESN'T** use this feature for user actions, so callbacks are not possible when using Interchain Abstract Accounts. If you are a `module` developer, check out the [Module Ibc](./module-ibc.md) section that allows for callbacks.

Abstract only uses Polytone Callbacks when:

- Registering a new Abstract IBC counterpart, to store the remote Polytone `proxy` caller.
- Creating remote accounts to be able to store the remote Abstract `Account` address locally.

#### Cross chain trace

Because accounts created across chains using the IAA protocol are controlled by an account located on a remote chain, the account that is calling the action needs to be related to the account on the remote chain.
This is done through the <a href="https://docs.rs/abstract-std/latest/abstract_std/objects/account/struct.AccountId.html" target="blank">AccountId</a> struct. The IBC-host module leverages the `AccountId::trace` field of this struct. An account is wether `AccountTrace::Local` or `AccountTrace::Remote`. When a PacketMsg is sent across an IBC channel, the account id is transformed on the receiving chain in the following manner:

- If it was `AccountTrace::Local` before transfer, it turns into an `AccountTrace::Remote` account with one chain in the associated vector being the chain calling the `PacketMsg` (`PacketMsg::client_chain`)
- If it was `AccountTrace::Remote` before transfer, it stays remote and the `client_chain` field is pushed to the associated vector.

This allows full traceability of the account creations and calls.
