# IBC capabilities

[![](https://docs.rs/abstract-sdk/badge.svg)](https://docs.rs/abstract-sdk) [![](https://img.shields.io/crates/v/abstract-sdk)](https://crates.io/crates/abstract-sdk)

Abstract provides IBC capabilities by default to every module. Within the framework, there are 2 ways to interact with other blockchains through IBC:

- Account IBC interaction
- Module IBC interaction

We start by giving an overview of the 2 mechanisms before diving in further on how you should use them as a developer and finally dive into the specific mechanism that are used by both capabilities.

## Overview

### Account IBC interaction

Abstract Account are able to send messages to other blockchains to execute actions. This allows any Abstract Account to create accounts on remote chains. This way, users create 1 account on their home chain and are able to execute any action on any IBC-connected chain. Use cases include:

- Executing actions on remote chains without having to care about the remote gas coin
- Cross-chain DCA strategies
- Cross-chain email
- ...
- Whatever permission-less application you can think of

Limitations:

- This capability doesn't allow modules to interact with one-another in a permissioned manner, because all messages sent are sent via the account directly and could be modified by the user.
- This capability doesn't allow IBC callbacks. This means that the result of IBC messages sent via this capability can't be used to trigger following actions directly.


[Learn more about Account Ibc interactions](../ibc/account-ibc.md)

### Module IBC interaction

This capability was added to Abstract to fill-in the limitations that the Account-IBC interactions presents. This capability allows modules to send messages directly to any other module present on remote chains. This allows permissioned use cases that were not possible with account-only interactions. Uses cases include:

- Distribued Interchain Name Service
- Cross-chain NFTs
- Cross-chain payments without cross-chain tokens
- ...
- Every IBC application can be built using Abstract !

After a message is successfully executed via IBC, callbacks can be triggered on the sending module to execute code depending on the result of the original message. Think od this mechanism as an asynchronous version of the <a href="https://docs.cosmwasm.com/docs/smart-contracts/message/submessage/#handling-a-reply" target="blank">`reply`</a> mechanism over IBC.

[Learn more about Account Ibc interactions](../ibc/module-ibc.md)
