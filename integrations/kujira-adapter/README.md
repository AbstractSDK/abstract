# Necessary ANS entries

This protocol needs the following entries inside the ANS contract to function.

## Oracle

No oracle contract, this is a Custom Query

## Vault

Where lended funds are deposited.

```rust
ContractEntry {
    protocol: "kujira".to_string(),
    contract: format!("vault/{}", lending_asset),
}
```

## Markets

Where Collateral are deposited and funds borrowed

```rust
ContractEntry {
    protocol: "kujira".to_string(),
    contract: format!("market/{}/{}", lending_asset, collateral_asset),
}
```
