# Abstract Name Service

The Abstract Name Service (or ANS in short) is an on-chain store of the most important data related to the blockchain it is deployed on. In Abstract it allows for chain-agnostic action execution and dynamic address resolution. These features allow both users and developers to interact with the blockchain in a more user-friendly way.

## ANS Architecture

The ANS is a smart contract that stores the following data:

- **Assets**: The most relevant assets on the local blockchain. 
- **Contracts**: Contracts related to certain protocols or applications that could be dynamically resolved. This could be used to store the address for an asset-pair for a dex. I.e. "osmosis/juno,osmo" could be resolved to the address of the osmosis pool that allows you to swap osmo for juno.
- **Channels**: IBC channel data to map a protocol + destination chain to a channel id. This allows for dynamic IBC transfers without having to know the channel id beforehand.
