# The Actor Model

```admonish info
Already a CosmWasm expert? Jump to [the next page](./3_account_abstraction.md) to learn about account abstraction!
```

The actor model is a computational model used in the design and implementation of the CosmWasm framework. It provides a deterministic execution environment by employing message-based communication between individual actors. In this model, each actor is an autonomous entity capable of managing its internal state, creating other actors (other smart-contracts), and send and receive structured messages.

The actor model plays a crucial role in designing the architecture of a smart-contract application within the CosmWasm framework.

## Benefits of the Actor Model

The application of the actor model in the CosmWasm framework provides the following benefits:

### 1. Encapsulation and Isolation

Each smart contract operates as an isolated actor, processing messages and state independently. This isolation prevents interference and unintended side effects between contracts, enhancing security and reducing the risk of vulnerabilities like re-entrancy attacks.

### 2. Sequential Message Processing

Contracts handle only one message at a time, ensuring deterministic execution and eliminating the need for call stacks. This approach prevents complex control flow issues and helps maintain the predictability of the contract's state. It also simplifies the mental model required to understand and reason about execution order in multi-contract interactions.

### 3. Controlled Interactions

When a contract wants to invoke another contract, it sends a message containing the necessary information. This controlled interaction allows for well-defined communication patterns, promoting modularity, and ensuring that contracts can safely collaborate without exposing their internal states.

### 4. Error Handling and Atomic Transactions

A transaction in CosmWasm can represent a state-transition that involves multiple sub-messages and spans multiple actors. If any sub-message on any actor encounters an error, the entire transaction is rolled back, reverting all state changes made within that transaction. This ensures atomicity and prevents inconsistent states in the contract
