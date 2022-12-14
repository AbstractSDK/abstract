# Abstract Module Versioning

A document detailing the Abstract module versioning.

## Module version dependencies

Every module (App/Extension) has a set of dependencies it can declare. The dependencies ensure a couple things:

- A module can only be installed when it's dependencies are installed.
- A module can not be un-installed as long as some other module depends on it.
- A module can not be upgraded if its dependents don't support the new version (major release)

## Version requirements

Version requirements are stored in the module itself and can be queried by the manager.

## Manager Version-Management Flow

### Installation

1. Call module factory to add the module.
2. On factory callback, query module dependencies and assert current state passes requirements.
3. Add dependents to dependency store.

### Migration

1. Retrieve new version number of module.
2. Load dependents of the to-be-migrated module and assert new version passes dependent requirements.
3. Remove old dependencies.
4. Update dependency version.
5. Add dependencies (there might be a new requirement)

### Uninstall

1. Load dependents of module and assert it is empty.
2. Query dependencies and remove self as dependent.
3. Uninstall the module.

## Notes on major releases

When a new major version of a module is released all the dependent modules should be upgraded first.
