# Abstract Modules Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](http://keepachangelog.com/)
and this project adheres to [Semantic Versioning](http://semver.org/).

## [Unreleased] - yyyy-mm-dd

### Added
  
### Changed

### Removed

### Fixed

## [0.20.0] - 2024-02-20

### Added

- Usage fee query for the dex adapter
  
### Changed

- Generate message now accepts `sender` instead of `proxy_addr`, allowing use of non-abstract addresses

### Removed

### Fixed

- Removed checks for manager in `fetch_data` for dex adapters, to allow using adapter requests by authorized addresses
