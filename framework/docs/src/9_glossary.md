# Abstract Glossary

These are some definitions used in our documentation:

## Abstract

A framework designed to simplify the development of decentralized applications in the Cosmos ecosystem. It offers tools
and infrastructure for composable smart-contract applications.

## Abstract Account

A unique entity within the Abstract framework that can have modules installed onto it, enabling various functionalities.

## Account

Short for Abstract Account.

## Abstract Account Console

A web-based interface that provides functionalities like account management, module management, name service, dev tools,
and delegations.

## Abstract APIs

Interfaces provided by Abstract to facilitate interactions between the frontend and the on-chain framework.

## Abstract Base

The foundational layer of the Abstract framework, upon which other functionalities and modules are built.

## Abstract Modules

Pre-built functionalities that can be installed onto an Abstract Account. They come in three types: App, Adapter, and
Standalone.

## Abstract Name Service (ANS)

An on-chain store that provides chain-agnostic action execution and dynamic address resolution.

## Abstract SDK

A toolbox for developers to create composable smart-contract APIs in the Abstract ecosystem. It provides a set of tools
and utilities to facilitate the creation and interaction of smart contracts.

## Abstract-Testing

A package that provides testing utilities for CosmWasm contracts, focusing on mocking and querying functionalities.

## Abstract.js

A JavaScript library designed to facilitate interactions with the on-chain Abstract framework.

## Account Abstraction

A concept where the Abstract Account acts as a layer abstracting the complexities of blockchain interactions, allowing
for a more user-friendly experience.

## Account Ownership

The concept that defines who has control and access rights over an Abstract Account. This can be a single entity (
Monarchy) or multiple entities (Multisig).

## Adapter

A type of Abstract Module that acts as an intermediary, translating and routing messages between Apps and external
services or protocols.

## API Objects

Rust structs in the Abstract SDK that expose specific smart-contract functionalities. They can be used if a contract
implements the required features/api traits.

## App

A type of Abstract Module designed to enable specific features or transform Abstract Accounts into standalone products.

## Cosmos

A decentralized network of independent, scalable, and interoperable blockchains. The Cosmos ecosystem is built on a set
of modular, adaptable, and interchangeable tools, with the Cosmos SDK being its foundational framework. Cosmos aims to
create an "Internet of Blockchains" where different blockchains can communicate and transact with each other seamlessly
through the Inter-Blockchain Communication (IBC) protocol.

## CosmWasm

A smart contract platform built for the Cosmos ecosystem. Within the Abstract framework, CosmWasm serves as the
underlying smart contract platform that powers the modular and composable functionalities of Abstract Modules. It allows
developers to write secure and interoperable smart contracts in Rust, which can then be integrated into the Abstract
ecosystem. By leveraging CosmWasm, Abstract ensures that its modules and applications are both scalable and compatible
with the broader Cosmos ecosystem.

## CW-Orchestrator

CW-Orchestrator is a scripting tool specifically designed to streamline interactions with, testing and deployment of
CosmWasm smart contracts.

## IBC-Host

A module that facilitates Inter-Blockchain Communication (IBC) within the Abstract framework, allowing for cross-chain
interactions.

## Integration Testing

Testing that involves deploying the contract and its dependencies to a mock environment to ensure they work together
correctly.

## JSON Schema Linking

Linking a module's JSON schema to the Abstract Registry to improve user experience for developers using the
module.

## Migration Update

A process within the Abstract framework that allows for the updating or upgrading of modules without compromising the
state or data.

## Mock Querier

A tool provided by the abstract-testing package to mock Smart and Raw queries for unit testing.

## Module Factory

A contract that allows the installation and management of Abstract Modules via the Account.

## Module Installation

The process of adding a module to an Abstract Account, specifying its parameters, and initializing it on a specific
network.

## Module Uploading

The process of compiling a module as a WASM binary and then uploading it to the desired network(s).

## Monarchy

A type of account ownership where a single entity has full control over an account.

## Move Update

A process that allows for the migration of an Abstract Account from one blockchain to another within the Cosmos
ecosystem.

## Multisig

A type of account ownership where multiple entities have control over an account, and a predefined number of them must
agree on actions taken.

## Namespace

A unique publishing domain for Abstract modules, associated with an Abstract Account. It's used to uniquely identify and
monetize modules.

## Raw Queries

Simple database key-value lookups without the computational aspect of smart queries.

## Rust

A systems programming language that focuses on performance, reliability, and productivity. Rust offers memory safety
guarantees by using a borrow checker to validate references. It's known for its "zero-cost abstractions," meaning
developers can write high-level code without sacrificing performance. Rust has gained popularity for blockchain and
smart contract development due to its safety features and efficient performance.

## Smart Queries

Queries that contain a message in their request and often involve computation on the queried contract.

## Registry

A contract that acts as a registry for all modules and accounts within the Abstract platform.