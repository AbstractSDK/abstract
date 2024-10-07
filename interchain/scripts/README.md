# Abstract Scripts

This crate contains scripts to deploy and interact with Abstract. This README contains multiple tutorials relate to Abstract deployments

## Deploying Abstract IBC

This is a unifying guide on how to deploy Abstract IBC between 2 chains.

### Deploy Abstract

The first step in deploying abstract is deploying Abstract on the 2 chains you want to connect.

### Deploy and Connect Polytone

Then you need to deploy and connect Polytone between the 2 chains. You should find a complete tutorial inside the [abstract-polytone repo](https://github.com/AbstractSDK/polytone/blob/abstract/deployment/DEPLOY.md).

This comes in 3 steps:

#### Deploy the Polytone contracts on each chain

Upload the polytone contracts (voice, note and proxy at least) on each chain. A script is available in the [abstract-polytone repo](https://github.com/AbstractSDK/polytone/blob/abstract/deployment/scripts/src/bin/upload_contracts.rs).

#### Instantiate the contracts on each chain

In order to create a one-directional connection, you need to instantiate the note on the local side and the voice on the remote side. A script is available in the [abstract-polytone repo](https://github.com/AbstractSDK/polytone/blob/abstract/deployment/scripts/src/bin/instantiate_chains.rs).

#### Channel creation

Now you can create a channel between the two contracts. The important parameters are:

- The channel needs to be `unordered`
- The channel version needs to be `polytone-1`

A more detailed explanation is available in the [abstract-polytone repo](https://github.com/AbstractSDK/polytone/blob/abstract/deployment/DEPLOY.md#channel-creation).

You have successfully a one-directional polytone connection. Don't forget to create the channel in the other direction to be able to have Abstract connect both ways !

#### (Optional) Testing the connection

You can now test the connection by sending an empty message for instance between the two chains! A script is available in the [abstract-polytone repo](https://github.com/AbstractSDK/polytone/blob/abstract/deployment/scripts/src/bin/verify_deployment.rs).

### Connect Abstract to the Polytone Deployment

#### State

In order to be able to read the polytone state (contract addresses) from the Abstract IBC deployment script, you need to publish polytone (either github or crates.io, or even use local deps if you don't want to publish).

#### Connecting

Now that your script has access to the polytone deployment, you can use the [Abstract IBC deployment script](./src/bin/connect_ibc.rs). This script does the following thin:

- Registers the polytone note AND the remote IBC Host inside the local IBC Client.
- Waits for a first message execution on the remote chain. This first message is not necessary for Polytone but allows us to register the remote proxy address inside the IBC Client
- Registers the remote proxy address on the remote IBC Host to whitelist execution
