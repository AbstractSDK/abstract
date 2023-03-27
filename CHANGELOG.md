# Abstract Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](http://keepachangelog.com/)
and this project adheres to [Semantic Versioning](http://semver.org/).

## [Unreleased] - yyyy-mm-dd

### Added

### Changed

### Fixed

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

- Fixed abstract-boot publishing

## [0.5.0] - 2022-01-08

### Added

### Changed

### Fixed

- Fixed wasming with `write_api` error in the `abstract-api` and `abstract-app`

## [0.5.0] - 2022-01-08

### Added

#### Module Factory

- unit testing

#### Ans Host

- `Config` query

#### Abstract SDK

- Better querying of app and api directly vs message construction

### Changed

- `PoolId` is now renamed to `PoolAddress` to avoid confusion with the Abstract Pool Id (and because it can be resolved
  to an address / id)

### Removed

- `construct_staking_entry` from `ContractEntry`, which had previously violated the SRP.

### Fixed
