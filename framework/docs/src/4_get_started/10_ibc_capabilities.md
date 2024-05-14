# IBC capabilities

[![](https://docs.rs/abstract-sdk/badge.svg)](https://docs.rs/abstract-sdk) [![](https://img.shields.io/crates/v/abstract-sdk)](https://crates.io/crates/abstract-sdk)

Abstract provides multiple IBC capabilities to every module. Within the framework, there are two ways to interact with other blockchains through IBC:

- Account IBC interaction
- Module IBC interaction

We start by giving an overview of these two mechanisms before diving in further on how you should use them as a developer and finally dive into the specific mechanism that makes them work.

## Overview

### Account IBC interaction

Abstract Account are able to send messages to other blockchains to execute actions. This allows any Abstract Account to create accounts on remote chains. This way, users create 1 account on their home chain and are able to execute any action on any IBC-connected chain. This kind of interaction can be likened to Cosmos's Interchain Account (ICA) functionality. Use cases include:

- Executing actions on remote chains without having to care about the remote gas coin
- Cross-chain DCA strategies
- Cross-chain email
- ...
- Whatever permission-less application you can think of

Limitations:

- This capability doesn't allow modules to interact with one-another in a permissioned manner. Because all messages are sent via the account directly they could be modified by the user. This means that the receiving module, on the other chain, can't be sure about the source of the message.
- Account execution doesn't allow for IBC callbacks. This means that the result of IBC message execution sent via this route can't be used to trigger following actions directly.

[Learn more about Account Ibc interactions](../ibc/account-ibc.md)

### Module IBC interaction

We developed this feature to Abstract to address the limitations present in the Account-IBC interactions. Module IBC allows modules to send messages directly to any other module present on a remote chains. This allows permissioned execution because the receiving module can verify and trust the origin of IBC packet. Uses cases include:

- Distribued Interchain Name Service
- Cross-chain NFTs
- Cross-chain payments without cross-chain tokens
- ...
- Every IBC application can be built using Abstract !

After a message is successfully executed via IBC, callbacks can be executed on the sender module to execute code depending on the result of the original message. You can think of this mechanism as an asynchronous version of the <a href="https://docs.cosmwasm.com/docs/smart-contracts/message/submessage/#handling-a-reply" target="blank">`reply`</a> mechanism over IBC.

[Learn more about Account Ibc interactions](../ibc/module-ibc.md)

### Recap

Abstract provides all the IBC-abstractions you need to build permissionless and permissioned IBC applications. Through the Module IBC mechanism, you can build meshed applications that interact with each other over different networks without having to worry about permissions, data structures, channel creation and maintenance, and all the other complexities that come with using IBC in CosmWasm.

IBC is a key feature of the Abstract framework and, as the ecosystem grows, we will continue to improve and expand the capabilities of our IBC offering.
