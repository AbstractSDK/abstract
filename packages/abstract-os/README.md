# Abstract

This package contains everything you need to interact and query with Abstract contracts.

This package contains:
* Abstract contract interface messages
* Abstract contract states
* Abstract objects (preferably use the re-exported version in Abstract-SDK)

```rust
// Import a contract Exec/Init/Query message
abstract_os::contract::ExecuteMsg
// Import a contract state lay-out
abstract_os::contract::state::STATE
// Import a state object
abstract_os::object::ProxyValue;
```

## Messages 
Each interface in individually commented.  

## States
Each state-layout is also individually covered

## Objects
Abstract Objects are used to store some kind of information along with helper functions. 
We provide these as imports so the data can be retrieved externally (using Raw Queries). 

If you want to use these objects in your own contracts we suggest using the abstract-sdk package.