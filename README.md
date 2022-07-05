# Abstract OS Architecture


![alt text](https://github.com/Pandora-OS/contracts/blob/main/architecture.png?raw=true)
## Core
Every OS start off with the core contracts. These contracts handle the most important business logic of the DAO, including fund management.

## Governance
The governance contract determines how the decision making withing the dao is handled.
- One owner (not really a DAO)
- Multisig
- Token weighted
- ...

## DApps
DApps are extentions to be installed on top of the core contracts.
### External DApps
These DApps handle the construction of messages and relaying of price information related to some external contract / service. Some examples are
- Anchor (+ aUST price)
- Terraswap
- Astroport
- Mirror (TODO)
- ...

### Internal DApps
These DApps add extra functionality to the core contracts. Some examples are
- Oracle
- Payroll
- Vault
- Custom factories
- ...

## Services
Services are contracts that offer some extra service to the DAO and remain a part of it. These include:
- Community Fund
- Emissions
- Vesting
- LP emissions / Proxy
- ...

## Perks
Perks are contracts that are not really part of the DAO infrastructure but are deployed for their one-time functionality. Some examples are
- Airdrop
- Liquidity Bootstrapping Pool
- ...

# Permissions
Permissions are handled on a per-dapp basis for interactions with the proxy. Changing configurations on the individual dapps is handled by the manager contract. This allows for flexible configuration of the dapps while still preserving the security guarantees.

Contract migration is handled by the manager contract which has special Admin right to do so. Other configuration rights are configured on a per-dapp basis.

The OS manager is Admin over all contracts. On instantiation the Module Factory is the default Admin which is changed after init.

# Deployment
Deploying the basic infrastucture is done though the following steps:

prerequisite:
1. Memory contract is deployed and instantiated
2. All required contracts have been uploaded and have a valid code-id

- Instantiate the version-control contract
- Add uploaded contract ids to version-control

Instatiating a new contract always relies on the version control contract to get the code_id.

The Manager contract is the root contract to control OS contract versions on a per-OS basis.

Now we'll create a new OS, this is done using the Factory contract

- Instantiate the manager contract and set OS-ID. The manager contract has a root_user address that gives permissions to change dapp parameters (like who's allowed to interact with that dapp)

- Factory has map with (Module,version) -> Binary for the standard configurations. The CW-Admin of each contract (allowed to change the code-id) is the manager contract.

- Add OS manager address to version-control
- Instantiate the Treasury contract
- Add it to the manager contract
- Instantiate the required governance (if applicable)
- Add whatever dApps and services you require through the version-control contract


# Manual Schema Generation
To generate the schemas for all the packages in this ws, run the following command. You may have to install [cargo workspaces(https://github.com/pksunkara/cargo-workspaces):
```bash
cargo install cargo-workspaces
```
When it is installed, run the following to generate schemas for each:
```bash
cargo ws exec --no-bail cargo schema
```
