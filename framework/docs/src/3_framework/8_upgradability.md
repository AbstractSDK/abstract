# Module Upgradability

Smart-contract migrations are a highly-debated feature in smart-contract development. Nonetheless Abstract believes it to be a powerful feature that allows for fast product iteration. In the spirit of crypto we've designed a system that allows for *permissionless software upgrades while maintaining trustlessness.*

## Module version storage

Permissionless software upgradeability is provided by a module version storage in the [version control contract](../platform/version_control.md). The mapping allows your Account to:

- Instantiate a module of the latest versions.
- Upgrade a module to a new version as soon as it's available.
- Provide custom modules to other users.
- Do all this without any third-party permissions.

There are two types of possible migration paths, although they appear the same to a user.

## Migration Update

Most module updates will perform a contract migration. The migration can be evoked by the root user and is executed by the manager contract. You can learn more about contract migrations in the CosmWasm documentation.

## Move Updates

In we outlined the reasoning behind our module classification system. More specifically we outlined why the API modules should not be migratable because it would remove the trustlessness of the system.

Therefore, if we still want to allow for upgradeable API's we need instantiate each API version on a different address. When a user decides to upgrade an API module, the abstract infrastructure moves his API configuration to the new addresses and removes the permissions of the old API contract.

Crucially, all other modules that depend on this API don't have to change any stored addresses as module address resolution is performed through the manager contract, similar to how DNS works!
