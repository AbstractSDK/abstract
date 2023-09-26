# Abstract Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](http://keepachangelog.com/)
and this project adheres to [Semantic Versioning](http://semver.org/).

## [Unreleased] - yyyy-mm-dd

### Added

- Install modules on account or Sub-account creation.
- Manager stores his sub-accounts and sub-accounts can register or unregister in case of ownership change.
- Query on module factory to see how much funds needs to be attached for installing modules.
- Version control on instantiation to the Apps alongside with registry traits.
- Instantiation funds added to module configuration, allowing modules to perform external setup calls.
- An `adapter_msg_types` similar to `app_msg_types`. This can be used to easily define the top-level entrypoint messages.

### Changed

- Updated fetch_data arguments of CwStakingCommand
- StakingInfoResponse now returns staking target(which is either contract address or pool id) instead of always staking contract address.
- Owner of the sub-accounts now Proxy, allowing modules to interact with sub-accounts.
- Install modules replaced install module method on module factory to reduce gas consumption for multi-install cases.
- Modified the account id structure. Each account is now identified with a unique ID and a trace. This is a requirement for Abstract IBC.
- Register Module(and Add Module) will now accept list of items, which reduces gas for multi-module install
- Removed the `CustomSwap` option on the dex adapter.
- Stake methods on cw-staking adapter now accept list, allowing users to do multi-stake/unstake/etc.
- Added must_use attribute on abstract sdk methods
- Renamed `abstract-(dex/staking)-adapter-traits` to `abstract-(dex/staking)-standard`

### Fixed

- Partially fixed cw-staking for Osmosis.
- Manager governance now changes only after new "owner" claimed ownership.
- Fixed and separated cw-staking and dex adapters for kujira.
- `ExecOnModule` calls now forward any provided funds to the module that is called.
- Manager queries of standalone module versions will now return version of the contract from the Version Control storage instead of error  

## [0.17.2] - 2023-07-27

### Added
- Neutron + Archway to registry

### Changed

### Fixed

## [0.17.1] - 2023-07-26

### Added

- Ability to set admin to native contracts during instantiation
- Query handler for module data
- Added neutron

### Changed

- Address of App/Adapter returned and set by default.

### Fixed

## [0.17.0] - 2023-07-05

### Added

- Ability to add module metadata.
- Ability to set an install fee for modules.
- Account interaction helpers

### Changed

- Removed the ability to claim multiple namespaces.
- It is now possible to replace a module code-id/address on testnets.

### Fixed

- Adapter execution from the manager with a provided proxy address is now allowed.

## [0.7.0] - 2023-02-15

### Added

### Changed

- Errors now need to implement `From<AbstractError>` and `From<AbstractSdkError>`

### Fixed

## [0.7.0] - 2023-02-01

### Added

### Changed

- Version Control `Modules` / `ModuleList`

### Fixed

## [0.5.2] - 2023-01-10

### Added

### Changed

### Fixed

- Fixed abstract-interface publishing

## [0.5.0] - 2022-01-08

### Added

### Changed

### Fixed

- Fixed wasming with `write_api` error in the `abstract-adapter` and `abstract-app`

## [0.5.0] - 2022-01-08

### Added

#### Module Factory

- unit testing

#### Ans Host

- `Config` query

#### Abstract SDK

- Better querying of app and adapter directly vs message construction

### Changed

- `PoolId` is now renamed to `PoolAddress` to avoid confusion with the Abstract Pool Id (and because it can be resolved
  to an address / id)

### Removed

- `construct_staking_entry` from `ContractEntry`, which had previously violated the SRP.

### Fixed
