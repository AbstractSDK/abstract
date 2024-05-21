# Bitsong Profile Account LifeCycle Workflow

## Smart Contract Infrastructure

Powering the smart contract account profiles for the Bitsong ecosystem is an infused fork of two main components:
- Abstract's Account Abstraction Framework
- Stargaze's Names Contracts

## New Contracts

### [Bs721-Profile](./contracts/account/bs721-profile)
This is the modified bs721 implementation for Bitsong Profiles. During ownership transfer, a msg is sent to the marketplace to update the owner saved to the marketplace state.

### [Profile Marketplace](./contracts/native/profile-marketplace)
The profile marketplace can only be deployed once per contract instantiation. A marketplace contract has a configuration of:
- `trading_fee_percent` a global % for trades
- `min_price` of profile names
- `ask_interval` timeout buffer on bids
- `factory` address of account factory to serve as profile minter 
- `collection` address of profile collection deployment

An owner of a profile-token may accept bids made by others on their profile token. Bids are made by sending tokens to escrow via the marketplace contract, and are kept track by the bidder address. 

## Modifications 
Below describes the modifications made to the contracts.

## [Account Factory](./contracts/native/account-factory)

### Creation of an Account 
![account](./images/create-account.png)
There is now an optional string value `bitsong_profile` provided when creating an account or even a sub account. All profile tokens are sent to the account proxy address when minted, and are in full control of each account. 

The profile name must be compatible with the same rules as Internet Domain Names. If there are fees required, they are validated during this contract call, and then a new profile nft is minted if the profile name as the token-id does not already exist. The proxy address is set as the owner for the marketplace contract state, as well as internally.

## Account Manager 
[Here](https://docs.abstract.money/3_framework/3_architecture.html#manager-contract) explains the modules primary functions. 

### Sending Msgs
![send](./images/send-msg-from-account-as-proxy.png)
The account manager will be the contract called to broadcast msgs from the smart contract accounts. This is handled by passing the msgs to the accounts proxy, by  calling the `exec_on_module` entry point.


## Account Proxy
[Here](https://docs.abstract.money/3_framework/3_architecture.html#proxy-contract) explains the modules primary functions. the entry points `execute_module_action` & `execute_module_action_response` expects msgs to come from internal list of registered addresses that can access the proxies function. `execute_module_action_response` is used when a response is needed to be handled calling the contract. 

# Full Framework Deployment Directions
![send](./images/deployment.png)
These directions compliment [abstract deployment scripts](../interchain/scripts/README.md) in order to deploy the custom contract versions for bitsongs specifications. 

## Option 1: Bash Scripts
You will need to have intalled & configured a network client binary `bitsongd` prior to running these bash scripts
## 1. Full Deployment 
These scripts:
-  uploads compiled contracts
-  instantiates framework 
-  configures framework to default format 

to compile:
```sh 
sh scripts/build_release.sh
```

to upload:
```sh
sh scripts/upload.sh
```
make sure you record down the code-id of each contract!

to deploy, make sure you populate the correct code_ids
```sh 
sh scripts/create_deployment.sh
```

## Option 2: Abstract Client 
The Abstract Client provides us will a full integration deployment script, run with cargo. To deploy to bitsong:
```sh 
# todo!()
```



