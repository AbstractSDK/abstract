# Necessary ANS entries

This protocol needs the following entries inside the ANS contract to function.

## Oracle

The price source for everything Mars

```rust
ContractEntry {
    protocol: "cavern".to_string(),
    contract: "oracle".to_string(),
}
```

## Market

Where everything stable related happens inside Cavern Protocol.

```rust
ContractEntry {
    protocol: "cavern".to_string(),
    contract: "market".to_string(),
}
```

## Overseer

Where LTV related queries are done inside Cavern Protocol

```rust
ContractEntry {
    protocol: "cavern".to_string(),
    contract: "overseer".to_string(),
}
```

## Custody contracts

Where collateral operations are executed. There is one contract per collateral asset.

```rust
ContractEntry {
    protocol: "cavern".to_string(),
    contract: format!("custody/{}", collateral_asset),
}
```
