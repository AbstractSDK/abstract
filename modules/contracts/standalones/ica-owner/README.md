# Interchain Account Owner Standalone module

## Description

The Interchain Account Owner Standalone module is used to demonstrate example of using abstract standalone module to control `cw-ica-controller` contract.

## Prerequisites

Tests of this module done with cw-orch-starship. Check out <https://orchestrator.abstract.money/interchain/quick-start.html#with-starship> for more details and Starship installation.

For fast setup see [Starship setup](starship/setup.md#starship-setup)

## Running tests

### Make sure starship is running

Starship commands intended to be used inside `starship/` directory

Start starship:

```bash
    make install
```

Once all nodes are up (see next section to check status) forward all the ports:

```bash
    make port-forward
```

To know the status of all the starship pods, use:

```bash
    kubectl get pods
```

Or for a watch version:

```bash
    watch kubectl get pods
```

### Start tests

```bash
    RUST_LOG=info cargo test --test starship -- --nocapture
```

Tests will use `~/.cw-orchestrator/starship-state.json` for the cw-orchestrator state to make sure it doesn't corrupt original state. It's safe to delete this file if you are using new starship infrastructure.

## Stopping starship

Stop starship:

```bash
    make stop
```

Delete Infrastructure:

```bash
    make delete
```

# Creating Connections using Hermes Relayer.

## Add the chain configurations

```bash
hermes config auto --output ~/.hermes/config.toml --chain 'testnets/archwaytestnet'  --chain 'testnets/dydxtestnet' 
```

## Add the keys

```bash
hermes keys add --chain "constantine-3" --mnemonic-file MNEMONIC_FILE
```

## Create the connection
```bash
hermes create connection --a-chain "constantine-3" --b-chain dydx-testnet-4
```

And note the result. 