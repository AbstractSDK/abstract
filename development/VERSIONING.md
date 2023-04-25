# Abstract Publishing & Versioning

This document describes the current versioning system for Abstract's contracts and packages. It is intended to be a living document that is updated as the project evolves. Discussions and suggestions are welcome.

The versioning setup is based on the dependency graph of the project, which is shown below:

> To generate the dependency graph run:  
>`cargo depgraph --dedup-transitive-deps --exclude abstract-boot-scripts,boot-core --build-deps | dot -Tpng > dep_graph.png`  
> *Ensure you have [graphviz](https://graphviz.gitlab.io/download/) installed.*

![Abstract's dependency graph](../docs/dep_graph.png)

Form the graph we can draw some conclusions:

1. `abstract-api` and `abstract-app` depend on `abstract-boot` and are the top-level packages within Abstract. Hence they can be independently versioned.
2. `abstract-boot` depends on all the contracts in the repository. The contracts should share their version with `abstract-boot`.

## Versioning

Most of the versioning information is contained within the workspace level [`Cargo.toml`](./Cargo.toml). The version defined in the `[workspace]` section is used as the version for all contracts in the workspace, as well as the lower level packages. The only exceptions are `abstract-api` and `abstract-app`, which can be versioned independently.

## Updating Dependencies

### Periphery

1. Update the version of Abstract in the integration-bundles repository.
2. Create a new release on the periphery repository (run `just publish`)

### Contracts

1. Upgrade the versions in the [`contracts`](https://github.com/AbstractSDK/contracts) repository to the new version via
   global find + replace (but be careful).
2. Run just publish to publish the packages to the new version.
3. Run just schema

### Modules

1. Upgrade the Abstract packages in the [`apis`](https://github.com/AbstractSDK/apis) repository to the new version in
   the base Cargo.toml
2. Run just publish
3. Upgrade the Abstract packages in the [`apps`](https://github.com/AbstractSDK/apps) repository to the new version in
   the base Cargo.toml
4. Run just publish

### Abstract.js

1. Run the ts-codegen script
