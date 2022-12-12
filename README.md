# Abstract OS 

<a href="https://codecov.io/gh/Abstract-OS/contracts" > 
 <img src="https://codecov.io/gh/Abstract-OS/contracts/branch/main/graph/badge.svg?token=FOIDUFYSCY"/> 
 </a>

<!-- ![alt text](https://github.com/Pandora-OS/contracts/blob/main/architecture.png?raw=true) -->
## Core
Every OS start off with the core contracts. These contracts handle the most important business logic of the DAO, including fund management.

## Governance
The governance contract determines how the decision making withing the dao is handled.
- One owner
- Multisig
- ...

## Modules
DApps are extentions to be installed on top of the core contracts.
### Extension's
These contracts handle the construction of messages and relaying of price information related to some external contract / service. Some examples are
- Junoswap
- Loop
- ...

### Add-ons
These contracts add extra functionality to the OS. Some examples are
- Oracle
- Payroll
- ETF
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
Changing configurations on an individual module is handled by the manager contract. This allows for flexible configuration of the dapps while still preserving the security guarantees.

Contract migration is handled by the manager contract which has special Admin right to do so. Other configuration rights are configured on a per-module basis.

The OS manager is Admin over all contracts.

# Deployment
Deploying the basic infrastructure is done though the following steps:

prerequisite:
1. AnsHost contract is deployed and instantiated
2. All required contracts have been uploaded and have a valid code-id

- Instantiate the version-control contract
- Add uploaded contract ids to version-control

Instantiating a new contract always relies on the version control contract to get the code_id.

The Manager contract is the root contract to control OS contract versions on a per-OS basis.

Now we'll create a new OS, this is done using the Factory contract

- Instantiate the manager contract and set OS-ID. The manager contract has a root_user address that gives permissions to change module parameters (like who's allowed to interact with that module)
- The CW-Admin of each contract (allowed to change the code-id) is the manager contract.

- Add OS manager address to version-control
- Instantiate the proxy contract
- Add it to the manager contract
- Instantiate the required governance (if applicable)
- Add whatever modules and services you require through the module factory contract

# Manual Schema Generation

To generate the schemas for all the packages in this ws, run the following command. You may have to install [cargo
workspaces(https://github.com/pksunkara/cargo-workspaces):

```bash
cargo install cargo-workspaces
```

When it is installed, run the following to generate schemas for each:

```bash
cargo ws exec --no-bail cargo schema
```

## Publishing

### Packages

To publish all the packages in the repo, execute the following steps:

1. Update all occurrences of the version in the Cargo.toml files
2. Publish using the following command

```bash
./publish/publish.sh
```

### Schemas

To publish the schemas to the [schema repo](https://github.com/Abstract-OS/schemas), run the following command:

```shell
cargo 
```

```bash
SCHEMA_OUT_DIR=$(cd ../schemas && echo "$PWD") \
VERSION=$(awk -F ' = ' '$1 ~ /version/ { gsub(/[\"]/, "", $2); printf("%s",$2) }' Cargo.toml) \
  cargo ws exec --no-bail bash -lc 'cargo schema && { outdir="$SCHEMA_OUT_DIR/abstract/${PWD##*/}/$VERSION"; echo $outdir; mkdir -p "$outdir"; cp -a "schema/." "$outdir"; }'
```

# Contract Migrate Ability

Migratable contracts are always a security risk. Therefore we'll outline all the migratable contracts and who's allowed
to do it here.

## Migratable

- Manager (root)
- Proxy (root)
- Add-ons (root)
- OS Factory (Abstract)
- Module Factory (Abstract)
- Version Control (Abstract)
- AnsHost

## Not Migratable
- Extensions
