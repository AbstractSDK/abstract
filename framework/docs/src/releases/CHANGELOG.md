# v0.x.x

## [Unreleased] - yyyy-mm-dd

### Added

- Fixed migration from xion accounts, must specify `code_id` field for such migration (because new code_id is not available inside migration function)

### Changed

- Account's `InstantiationMsg` field `owner` is optional now and defaults to AbstractAccount(account_address)

### Removed

## [0.24.1] - 2024-10-25

- Added `PfmMemoBuilder` API for building middleware forwarding memo
- Added `HookMemoBuilder` API for building wasm ibc hook memo
- `execute_with_funds` to Executor to attach funds to execution.
- `stargate` feature for abstract-app, abstract-standalone and abstract-adapter packages.
- New module type: `Service`, behaves the same as Native, but can be registered by any namespace.
- `AbstractClient`: `service` to get api of Service module
- `CustomExecuteHandler` To improve support for fully custom execute messages on Apps or Adapters
- `balance` method for `AnsHost` to query balance of `AssetEntry`
- `AbstractInterchainClient` to simplify Abstract deployments across multiple chains

### Changed

- **Merged `proxy` and `manager` contracts into `account`.
- Deployments now use pre-determined addresses. These addresses are hardcoded in the contracts.
- Ibc related renaming to add more consistency in namings
- Account action on executor takes `impl IntoIter<Item = impl Into<AccountAction>>` instead of `Vec<AccountAction>`
- Native contracts now have pre-compiled addresses. This removes the need for storing addresses in an on-chain state.
- Removed `UpdateConfig` endpoints from most native contracts and `App`/`Native` bases.
- Minified the storage namespaces and made them available via constants
- Version Control renamed to registry
- `registry::QueryMsg::Account` was changed to `registry::QueryMsg::Accounts` for simultaneous queries
- Added `registry::QueryMsg::AccountList` for paginated account queries
- Simplified the implementations of KeyDeserialize, PrimaryKey and Prefixer traits for  `AssetEntry`, `DexAssetPairing`, `ModuleInfo`, `ModuleVersion`. Used the base tuple implementation instead
- Removed `install_on_sub_account` for client, replaced with explicit sub_account creation

#### Abstract Client

- `with_modules` method for Account Builder to add list of modules to install (`ModuleInstallConfig`)
- `query_module` method for Account to query given module on account without retrieving `Application` object
- `module_installed` method for Account that returns `true` if module installed on account
- `module_version_installed` method for Account that returns `true` if module of this version installed on account
- `address` method for Account to get address of account. Result of this method is the same as calling `proxy`
- `enable_ibc` added to Account builder.
- `module_status` on AbstractClient that returns current status of the module.
- `install_on_sub_account` now defaults to `false` in Account Builder
- `Publisher` will check if dependencies of the module is registered in version control to the chain before publishing.

### Removed

- Receive endpoints from abstract Modules
- Value calculation logic from proxy contract.
- `cw-semver` dependency removed
- Manager no longer able to migrate pre `0.19` abstract adapters
- Account Factory contract
- Unused `DepositManager` and `PagedMap` objects from abstract-std

### Fixed

- Abstract Client: If Account Builder retrieves account now it will install missing modules from the builder instead of ignoring them

## [0.23.0] - 2024-07-16

### Added

- Abstract Client: Added a `claim_namespace` function to facilitate claiming a namespace after account creation
- Version Control interface: `approve_all_modules_for_namespace` to approve any pending modules by given "namespace"
- IBC module to module queries and API.
- Abstract Interface: Added helpers to create abstract IBC connections (with open-sourced cw-orch-interchain)
- Ability to send multiple query messages through IBC simultaneously
- New module type `abstract-standalone` for standalone contracts.
- Abstract Client: added `execute_on_manager` helper method
- Abstract Client: Exposed `IbcClient` object under `AbstractClient::ibc_client()`
- Abstract Client(feature "interchain"): `connect_to` to create abstract IBC connections
- Abstract Client(feature "interchain"): `RemoteApplication` and `RemoteAccount` objects that replicate `Application` and `Account` functionality in interchain environment
- Abstract Account: Added an `upgrade` helper to upgrade an account step by step (going through all necessary versions)
- IBC Client: Apps and Adapters checks that IBC Client is dependency of the module inside ibc_callback and module_ibc handlers
- Ibc Client: Module to module actions now checks if app have ibc_client installed to ensure account can receive ibc callback
- Helpers to simply connecting Abstract instances through IBC and reduce the setup boilerplate
- `register_in_version_control` added to the `abstract_interface::Abstract` for registering new versions of native contracts in Version Control
- Registration migrated native contracts to Version Control in `abstract_interface::Abstract::migrate_if_version_changed` method
- New governance type `NFT` which allows an account to be owned by an NFT.

### Changed

- Manager will try to check dependencies on standalone modules.
- Accounts with local sequence 2147483648..u32::MAX are allowed to be claimed in any order
- IBC Callback and IBC module to module endpoints now have decomposed variables (sender, msg and callback)
- IBC Callback messages are now mandatory and renamed to `callback`
- Removed IBC callback IDs
- Renamed `CallbackInfo` to `Callback`
- Ibc API: Where applicable - accept `ChainName` instead of `String` to add clarity for the user
- Standalones and IBC Client no longer added to proxy whitelist
- IBC client and host now migrated only if version is not breaking and deployed otherwise
- `cw-ownable` got replaced with `cw-gov-ownable` for manager contract
- Renamed `ChainName` to `TruncatedChainId`
- IBC Client: `send_funds` accepts optional `memo` field for every Coin attached
- Bump cw-orch to `0.24.0`

### Removed

- Accounts with local sequence 0..2147483648 cannot be predicted
- Ibc Callback handler no longer includes `MessageInfo` as sender is always ibc_client and funds are empty
- Account Factory no longer stores ibc-host, instead it queries VersionControl to assert caller matches stored to the one in version control
- `governance_details` from `manager::AccountInfo`
- Removed `update_factory_binary_msgs` endpoint from module factory
- Removed `propose_ownership` method on manager, everything done through `update_ownership` instead

### Fixed

- Abstract Client: Fixed contract address collision for same apps that are on different accounts
- abstract_interface deploy methods: Fixed a bug where it was not possible to propose uploaded contract(saved in cw-orch state)
- abstract_interface deploy methods: Checks both registered and pending modules instead of only registered

## [0.22.1] - 2024-05-08

### Added

- `state.json` now included in binary in release mode, allowing using binaries on a different environment than it's been built.
- `module_instantiate2_address_raw` for `AbstractClient`, allowing to install a different version than the dependency version.
- Added helper functions `assert_registered` and `is_registered` to the ANS client API.
- Added method `module_info` for querying and verifying wether an address is a module to the ModuleRegistry API.
- Added default IBC-Client installation on remote modules inside Client and Account interfaces
- Send multiple message simultaneously through IBC

### Changed

- Renamed `account_id` to `expected_account_id` for `abstract_client::AccountBuilder` for clarity
- Namespace claiming on mainnet is now permissioned.
- Renamed `version_control::Config::allow_direct_module_registration_and_updates` field to `security_disabled`.
- Renamed `request` to `execute` in adapter and apps APIs
- Updated to cw-orch 0.22 and cw-orch-core stabilization to 1.0.0

### Removed

- unused `custom_swap` of `DexCommand`
- Send multiple messages to multiple IBC connected chains in one manager message. 
- `interface` feature from all of the packages

### Fixed

## [0.21.0] - 2024-02-20

### Added
  
- Added a `.execute` method on the AuthZ API to execute `CosmosMsg` types on behalf of a granter.
- Add IBC helpers to account client.
- Abstract Client builder: register dexes on ANS
- `.sub_accounts` method on `Account` for getting Abstract Client Sub Accounts
- Publish adapter method of Abstract Client Publisher now returns Adapter object
- Added a `.account_from` method on the `AbstractClient` for retrieving `Account`s.
- Creating Sub Account from `AbstractClient` Account builder.
- Installing apps and adapters for `AbstractClient` Account builder
- Attaching funds to account creation on `AbstractClient` Account builder
- Added `unchecked_account_id` method on version control.
- Ability to provide expected local AccountId
- Reinstallation of the same version of an app is now disabled
- `.authorize_on_adapters` method on `Application` for authorizing application on adapters
- Added method to assign expected `.account_id` for Abstract Client Account builder
- `.next_local_account_id` for `AbstractClient` to query next local account sequence
- `.module_instantiate2_address` for `AbstractClient` to get predicted address

### Changed

- Updated UsageFee api to use `Address`, instead of `Api` + unchecked address
- Tests now use `MockBech32` due to use of instantiate2.

### Removed

### Fixed

- Added a validation on `account_id` method on version control.
- Creating sub-account from account factory is restricted. Use Create Sub Account method of the manager instead

## [0.20.0] - 2024-01-24

### Added

- `AppDeployer` and `AdapterDeployer` now take a `DeployStrategy` field.
- `Astrovault` integrated into dex and cw-staking adapters
- `AuthZ` API added
- Interchain Abstract Accounts can now be created!
- Added snapshot tests
- Method `query_account_owner()` for Apps Admin object
- Query `registered_dexes` for `AbstractNameServiceClient`
- Query `top_level_owner` for manager and apps(as base query)
- Support of `ConcentratedLiquidity` pool type for swaps. Stake/unstake currently not supported
- Account namespace is unclaimed after `Renounce`
- Resolve trait for `cw-orch` `AnsHost` interface

### Changed

- `is_module_installed` moved from `Manager` to `Account`.
- `account_id()` method of `AccountRegistry` is now exposed.
- Allow module-id to be passed in as a valid authorized address when allowing new addresses on adapter contracts.
- `BaseInstantiateMsg` is now removed from install app API, now only `ModuleMsg` should be provided.
- `Modules`, `Manager` and `Proxy` are now instantiated via instantiate2 message.
- `FeeGrant` API updated.
- Bump `cw-orch` to `v0.18`.
- Top level account owner now has admin privileges on the apps and adapters
- Multiple `AbstractAccount`s now don't overlap
- Top level account owner can now claim pending sub-accounts directly
- `Clearable` helper type was added to the messages where clearing optional state could be useful
- Only incremental version migration of modules allowed (0.10 -> 0.11 is allowed but 0.10 -> 0.12 not because it skips 0.11)
- Module `tag_response` and `custom_tag_response` no longer require `Response` as an argument as well as renamed to `response` and `custom_response` respectively.
- Having sub accounts will prevent you from `Renounce`
- Version Control `Namespace` query now doesn't return an error when namespace is unclaimed
- `NamespaceResponse` type updated to be able to represent claimed and unclaimed namespace

### Removed

- `DepositMsgs` removed (now `deposit()` returns `Vec<CosmosMsg>`)
- Abstract removed from the fields where it's redundant
- InstantiateMsg is now removed from the install_adapter API
- Removed `wasm_smart_query` helper, since it's accessible from `Querier` object
- Removed Adapter base `Remove` action

### Fixed

- Namespace registration fee fixed
- Version Control smart query now returns Version Control config instead of factory address
- Sub accounts now unregister themselves on owning manager if renounced

## [0.19.0] - 2023-09-26

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
