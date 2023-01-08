# Abstract-OS Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](http://keepachangelog.com/)
and this project adheres to [Semantic Versioning](http://semver.org/).

## [Unreleased] - yyyy-mm-dd

### Added

### Changed

### Fixed

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
