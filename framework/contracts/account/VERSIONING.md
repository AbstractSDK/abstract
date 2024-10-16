# Abstract Module Versioning

A document detailing the Abstract module versioning.

Instead of storing dependencies we store dependents for each module. So if a module A depends on B and C, then B and C will have A as a dependent.

`DEPENDANTS: Map<ModuleId, Vec<ModuleId>>`

## Module version dependencies

Every module (App/Adapter) has a set of dependencies it can declare. The dependencies ensure a couple things:

- A module can only be installed when it's dependencies are installed.
- A module can not be un-installed as long as some other module depends on it. (entry for the module in the dependants map is not empty)
- A module can not be upgraded if its (possibly also upgraded) dependents don't support the new version.

## Version requirements

Version requirements are stored in the module itself and can be queried.

## Manager Version-Management Flow

### Installation

1. Call module factory to add the module.
2. On factory callback, query module dependencies and assert current state passes requirements.
3. Add dependents to dependency store.

### Migration

1. Retrieve new version number of module. (if not provided)
2. Load dependents of the to-be-migrated module and assert new version passes dependent requirements.
    - Exclude any modules that are being migrated before. This ensures that the migration is not blocked by a module that is being migrated.
3. Migrate all the modules that have applied for a migration.

Then for each module that was migrated:

1. Remove self as a dependent when it no longer applies. I.e. the module depended on a module A but no longer does after the migration.
2. Add self as a dependent when it applies. I.e. the module did not depended on a module B but does after the migration.
now what could happen is that the new version of the module
2. Update dependency version.
3. Add dependencies (there might be a new requirement)

### Uninstall

1. Load dependents of module and assert it is empty.
2. Query dependencies and remove self as dependent.
3. Uninstall the module.

## Notes on major releases

When a new major version of a module is released all the dependent modules should be upgraded first.
