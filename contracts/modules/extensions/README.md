# ExtensionS 

Extensions are generic interfaces for on-chain services/dApps. 

Each extension is non-migratable and holds an optional entry of white-listed addresses linked to a proxy address. 

When a user calls on the extension it will first check its internal map for an entry. Only the Manager contract can add/remove permissions for executing on a proxy contract.

If no user is registered then the contract will try to verify if the sender is a Manager. If not, the execution will fail. If the sender is a registered manager then the requested msg will be forwarded to the linked proxy contract.

- No non-whitelisted user can execute extension calls on proxy contracts
- The root user can execute extension operation on the proxy contract trough the Manager contract.
- The OS owner can optionally whitelist addresses to reduce gas overhead.