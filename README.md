# DAO-OS

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


# Deployment
Deploying the basic infrastucture is done though the following steps:

prerequisite: 
1. Memory contract is deployed and instantiated
2. All required contracts have been uploaded and have a valid code-id

- Instantiate the version-control contract
- Instantiate the manager contract and set OS-ID
- Add OS manager address to version-control
- Add uploaded contract ids to version-control

Now we'll create a new OS, this is done using the Factory contract

- Instantiate the Treasury contract
- Add it to the manager contract
- Instantiate the required governance (if applicable)
- Add whatever dApps and services you require through the version-control contract