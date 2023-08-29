# Abstract Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](http://keepachangelog.com/)
and this project adheres to [Semantic Versioning](http://semver.org/).

## [Unreleased] - yyyy-mm-dd

### Added
- Install modules on account or Sub-account creation

### Changed
- Updated fetch_data arguments of CwStakingCommand
- Owner of the sub-accounts now Proxy, allowing modules to interact with sub-accounts

### Fixed
- Partially fixed cw-staking for Osmosis

## [0.17.2] - 2023-07-27

### Added
- Neutron + Archway to registry

### Changed

### Fixed

## [0.17.1] - 2023-07-26

### Added

- Ability to set admin to native contracts during instantiation
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
