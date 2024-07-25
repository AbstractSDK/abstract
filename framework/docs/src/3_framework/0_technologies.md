# Technologies

In this section, we'll briefly introduce you to the tech-stack that Abstract depends on. We'll do this from a top-down approach, starting with the smart-contract framework.

```admonish info
Already familiar with the stack? Jump to [the next page](./1_abstract_sdk.md) to start learning about the Abstract SDK.
```

## CosmWasm

The Abstract SDK and cw-orchestrator are tightly integrated with
the <a href="https://cosmwasm.com/" target="_blank">CosmWasm</a> smart-contract
framework.

<a href="https://cosmwasm.com/" target="_blank">CosmWasm</a> is a smart contract framework that is built on Rust and WebAssembly (WASM) to perform deterministic compute suitable for decentralized systems. It was born out of a desire to outgrow Solidity's shortcomings.

It is the only smart contract platform for public blockchains that has seen heavy adoption and stress-testing outside of the EVM ecosystem.

<!-- <details>
  <summary>Learn more</summary>

  ### The Actor Model

  The actor model is a computational model used in the design and implementation of the CosmWasm framework. It provides a
  deterministic execution environment by employing message-based communication between individual actors. In this model,
  each actor is an autonomous entity capable of managing its internal state, creating other actors (other
  smart-contracts),
  and send and receive structured messages.

  The actor model plays a crucial role in designing the architecture of a smart-contract application within the CosmWasm
  framework.

  #### Benefits of the Actor Model

  The application of the actor model in the CosmWasm framework provides the following benefits:

- **Encapsulation and Isolation**: Each smart contract operates as an isolated actor, processing messages and state
    independently. This isolation prevents interference and unintended side effects between contracts, enhancing security
    and reducing the risk of vulnerabilities like re-entrancy attacks.

- **Sequential Message Processing**: Contracts handle only one message at a time, ensuring deterministic execution and
    eliminating the need for call stacks. This approach prevents complex control flow issues and helps maintain the
    predictability of the contract's state. It also simplifies the mental model required to understand and reason about
    execution order in multi-contract interactions.

- **Controlled Interactions**: When a contract wants to invoke another contract, it sends a message containing the
    necessary information. This controlled interaction allows for well-defined communication patterns, promoting
    modularity,
    and ensuring that contracts can safely collaborate without exposing their internal states.

- **Error Handling and Atomic Transactions**: A transaction in CosmWasm can represent a state-transition that involves
    multiple sub-messages and spans multiple actors. If any sub-message on any actor encounters an error, the entire
    transaction is rolled back, reverting all state changes made within that transaction. This ensures atomicity and
    prevents
    inconsistent states in the contract.

</details>   -->

To learn more about CosmWasm, check out its <a href="https://book.cosmwasm.com/" target="_blank">official
documentation</a>.

### CosmWasm Coming from EVM

There are a few key differences between the EVM and CosmWasm that you should be aware of. The most important one is that instances of contracts and the code that they run against are two different concepts in CosmWasm. This means that you can have multiple instances of the same contract code running at the same time, each with their own state. This is not possible in EVM, where the contract code and the contract instance are the same thing.

This is an important difference to be aware of when we talk about migrations further in our documentation.

```admonish summary
Migrations are a key feature of CosmWasm. They allow you to upgrade a contract's code while retaining the state of the contract.
```

A migration doesn't delete the code that was previously running for a contract. Code (a WebAssembly binary) is referred to by code-ids and contracts run against a specific code-id and get their own address space (and state) when they are instantiated. Hence migrations just update the code-id that a contract uses to run. I.e. The contract keeps its address and state but now runs on a different code-id (binary).

> If you're looking for a more in-depth comparison go read this <a href="https://medium.com/cosmwasm/cosmwasm-for-ctos-f1ffa19cccb8" target="_blank">article</a> by the creator of CosmWasm.

## Rust

Diving one level deeper, we have the Rust programming language. While theoretically any language can be compiled to WebAssembly, Rust is the only language that is officially supported. This is because Rust is a systems programming language that is designed to be fast and extremely safe.

Learn more about Rust <a href="https://www.rust-lang.org/" target="_blank">here</a>, or learn by doing
the <a href="https://github.com/rust-lang/rustlings/" target="_blank">rustlings</a>.

## Typescript

You need to have a basic understanding of <a href="https://www.typescriptlang.org/docs/" target="_blank">Typescript</a> to use
our front-end library.

You can learn about Javascript <a href="https://developer.mozilla.org/en-US/docs/Learn/Getting_started_with_the_web/JavaScript_basics" target="_blank"> here</a>.
