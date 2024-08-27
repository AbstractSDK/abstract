# CosmWasm Staking

An Abstract Adapter module that handles staking and unbonding interactions with LP staking providers. 

## Naming Convention

In order to easily identify and relate contracts to on-chain addresses we follow the following conventions:

- *Staking AssetEntry*: an `AssetEntry` of the token that is stakeable. For LP tokens this is formatted as `{provider}/{asset_pair}`.
    -    Ex: `osmosis/cosmoshub>atom,juno>juno`
- *Staking AddressEntry*: a `ContractEntry` that is formatted as **{provider}:staking/{staking_asset_entry}**
    - Ex: `osmosis:staking/osmosis/cosmoshub>atom,juno>juno`



# CosmWasm Staking Adapter Module

The CosmWasm Staking Adapter Module provides a unified interface to interact with various protocols offering staking services to token holders. This can go from LP-staking to governance token locking. By abstracting the differences between various staking protocols, it allows developers to interact with any LP staking protocol using a standard interface, streamlining the development process and ensuring compatibility across various staking platforms.

## Features

- **Stake**: Deposit (and lock) assets inside a staking protocol
- **Unstake**: Remove (and unlock) assets from staking protocol. In protocols with a locking period after unstaking, this simply triggers the unbonding process
- **Claim Rewards**: Claims the rewards associated with locking your assets. In protocols with a locking period on rewad claim, this simply triggers the unbonding process
- **Claim**: Claim matured unbonding claims (from unstake of Claim rewards)

## Supported Staking Providers

The following Staking Providers are currently supported:

- Osmosis (Osmosis)
- Astroport (Neutron, Terra, Injective, Sei)
- Kujira (Kujira)
- Astrovault (Archway)
- Wyndex (Juno)

If you would like to request support for an additional Staking Provider, please create a GitHub issue or reach out to us on Discord.

## Installation

To use the CW-Staking Adapter Module in your Rust project, add the following dependency to your `Cargo.toml`:

```toml
[dependencies]
abstract-cw-staking = { git = "https://github.com/AbstractSDK/abstract.git", tag="<latest-tag>", default-features = false }
```

## Usage with the Abstract SDK

```rust
// Retrieve the adapter interface
use abstract_sdk::{AdapterInterface, core::objects::LpToken};
use abstract_cw_staking::{msg::StakingAction, msg::StakingExecuteMsg, CW_STAKING};
...

let provider = "osmosis".to_string();
let lp_asset = LpToken::new(provider, vec!["cosmoshub>atom".into(), "juno>juno".into()]));

let adapters = app.adapters(deps);  
   
let stake_msg = adapters.request(  
    CW_STAKING,  
    StakingExecuteMsg {
        provider,
        action: StakingAction::Stake {  
            assets: vec![AnsEntryConvertor::new(lp_asset).ans_asset()],  
            unbonding_period: None,
        },
    },
)
```

## Why Use the Cw Staking Adapter?

### Simplified Development
By using the Cw Staking Adapter, developers can bypass the intricacies of each individual Staking Provider. This means less time spent on understanding and integrating with each staking provider's unique API, and more time focusing on building core functionalities.

### Flexibility
Using Abstract adapters ensure that your application remains flexible. If a new Staking Provider or use-case emerges or if there are changes to an existing one, your application can easily adapt without undergoing major overhauls.

### Use Cases
- **Rapid Prototyping**: Quickly build and test applications on top of various staking providers without the need for multiple integrations.
- **Cross-Dex Applications**: Build applications that leverage multiple staking providers simultaneously, offering users more options and better rates.
- **Future-Proofing**: Ensure your application remains compatible with future staking solutions that emerge in the Cosmos ecosystem.

## Documentation

- **CW-Staking Interface**: For a detailed look at the cw-staking interface, refer to the [Rust trait interface](https://github.com/AbstractSDK/abstract/blob/bcf26f2f446478fd2825de5b187321dc9a626341/modules/contracts/adapters/cw-staking/src/api.rs#L43).
- **Adapters Documentation**: Comprehensive information about adapters can be found in the [official documentation](https://docs.abstract.money/framework/module_types.html#adapters).

## Contributing

If you have suggestions, improvements, new Staking Providers, or want to contribute to the project, we welcome your input on GitHub.
