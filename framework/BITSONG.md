# Bitsong Profile Account LifeCycle Workflow

## Smart Contract Infratstructure

Powering the smart contract account profiles for the Bitsong ecosystem is an infused fork of two main components:
- Abstract's Account Abstraction Framework
- Stargaze's Names Contracts

## Modifications 
Below describes the modifications made to the contracts.

## Profile Marketplace 
The profile marketplace can only be deployed once per contract instantiation. A marketplace contract has a configuration of:
- `trading_fee_percent` a global % for trades
- `min_price` of profile names
- `ask_interval` for bid increments on profile names
- `factory` address of account factory to serve as profile minter 
- `collection` address of profile collection deployment

## Account Factory 

### Creation of an Account 
There is now an optional string value `bitsong_profile` provided when creating an account or even a sub account. All profile tokens are sent to the account proxy address when minted, and are in full control of each account. The profile name must be compatible with the same rules as Internet Domain Names. If there are fees required, they are validated during this contract call, and then a new profile nft is minted, with the profile name as the token-id, with the proxy address of an account as the owner , and then finally the profile marketplace is then updated with the token-id & minter.


## Bs721-Profile 
This is the modified bs721 implementation for Bitsong Profiles. During a token transfer, a msg is sent to the marketplace to update the owner saved to the marketplace state.

# Full Framework Deployment Directions
These directions compliment [abstract deployment scripts](../interchain/scripts/README.md) in order to deploy the custom contract versions for bitsongs specifications. 

## 1. Store Code On-Chain
```sh
# todo!
```
## 2. Instantiate Infra-Contracts
```sh
# todo!
```

## 3. Instantiate Core Account Contracts
```sh
# todo!
```

## 4. Configure Contracts 
```sh
# todo!
```




