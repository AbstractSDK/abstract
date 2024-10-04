# Account Creation

As was covered in the section on Account Architecture an `Account` is just a single smart-contract that can act as a wallet. This means that creating an `Account` is as simple as instantiating a smart-contract.

## Account Parameters

Accounts are created by instantiating an `Account` contract that has been registered with the `registry` contract. The `registry` contract is a contract that keeps track of all the modules and account code-ids and addresses on the Abstract platform.

The `Account` contract takes the following parameters:

```rust,ignore
{{#include ../../../packages/abstract-std/src/account.rs:init_msg}}
```

The account can be instantiated using the normal `instantiate` function or `instantiate2`, which allows you to claim deterministic addresses.

During the instantiation process the account will self-register on the `registry` contract, making it queryable by its account-id.
