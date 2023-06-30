# Abstract

This package contains everything you need to interact with Abstract contracts.

This package contains:
* Abstract contract interface messages
* Abstract contract states
* Abstract objects (preferably use the re-exported version in [Abstract-SDK](https://crates.io/crates/abstract-sdk))

## Usage
```rust
// Import a contract Exec/Init/Query message
abstract_core::contract::ExecuteMsg
// Import a contract state lay-out
abstract_core::contract::state::STATE
// Import a state object
abstract_core::object::ProxyValue;
```

## Messages 
Each interface in individually commented.  

## States
Each state-layout is also individually covered

## Objects
Abstract Objects are used to store information along with helper functions. 
We provide these as imports so the data can be retrieved externally (using Raw Queries). 

If you want to use these objects in your own contracts we suggest using the abstract-sdk package.