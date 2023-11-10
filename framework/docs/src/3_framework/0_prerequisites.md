# Prerequisites

In this section, we'll cover the prerequisites for using the Abstract Platform and its products.

## Rust

The Abstract SDK is built on top of the <a href="https://cosmwasm.com/" target="_blank">CosmWasm</a> smart-contract
framework, which is written in <a href="https://www.rust-lang.org/" target="_blank">Rust</a>.
Therefore, it's important to have a basic understanding of Rust.

Rust is a modern and user-friendly programming language that shines in both performance and safety. Its innovative
features allow developers to write reliable and efficient code, making it a great choice for smart contracts.

Learn more about Rust <a href="https://www.rust-lang.org/" target="_blank">here</a>, or learn by doing
the <a href="https://github.com/rust-lang/rustlings/" target="_blank">rustlings</a>.

```admonish info
Already a CosmWasm expert? Jump to [the next page](./3_account_abstraction.md) to learn about account abstraction!
```

## CosmWasm

Some of the products such us the Abstract SDK and CW-Orchestrator are built on top of
the <a href="https://cosmwasm.com/" target="_blank">CosmWasm</a> smart-contract
framework. Abstract simplifies complexity within CosmWasm, but it's still important to understand the basics.

If you are already familiar with CosmWasm, you can skip this section.

### What's CosmWasm?

CosmWasm is a smart contract platform focusing on security, performance, and interoperability. It is the only smart
contracting platform for public blockchains with heavy adoption outside the EVM world.

#### Key Points

- Deeply integrated in many Cosmos blockchains
- Type-safe development and powerful testing with Rust
- Native supports for powerful IBC communication
- Compatible with many Rust libraries (i.g. decimal math)
- Uses highly optimized Web Assembly runtime
- Welcoming (and diverse) dev community

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

To learn more about CosmWasm, check out the <a href="https://book.cosmwasm.com/" target="_blank">official
documentation</a>.

## Javascript

You need to have a basic understanding of <a href="https://www.javascript.com/" target="_blank">Javascript</a> to use
out front-end library Abstract.js, which is used to
interact with the on-chain Abstract framework.

You can learn about Javascript <a href="https://developer.mozilla.org/en-US/docs/Learn/Getting_started_with_the_web/JavaScript_basics" target="_blank">
here</a>.
