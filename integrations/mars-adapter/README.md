# Necessary ANS entries

This protocol needs the following entries inside the ANS contract to function.

## Oracle

The price source for everything Mars

```rust
ContractEntry {
    protocol: "mars".to_string(),
    contract: "oracle".to_string(),
};
```

## Red Bank

Where everything happens inside Mars Protocol

```rust
ContractEntry {
    protocol: "mars".to_string(),
    contract: "red-bank".to_string(),
};
```
