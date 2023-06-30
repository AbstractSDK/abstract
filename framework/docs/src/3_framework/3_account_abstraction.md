# Account Abstraction

Account abstraction is a new concept that is making headlines on blockchain and smart-contract platforms. It's a popular subject because it is designed to streamline how users interact with decentralized applications (dApps). The fundamental idea is to abstract away the complexities of blockchain interactions and provide a user-friendly, secure interface for using and managing applications.

In traditional blockchain interactions, a transaction is typically initiated by a users directly signing some data with their private key and transmitting that to the blockchain for validation. Account abstraction simplifies this process by making the transaction initiation and validation programmable. Essentially, it allows the transaction logic to be customized within a smart-contract, vastly extending the scope of UX possibilities.

```admonish info
See [EIP-4337](https://eips.ethereum.org/EIPS/eip-4337) to read about account abstraction in the Ethereum ecosystem.
```

This concept of account abstraction, when implemented correctly, can provide numerous benefits:

1. **Improved User Experience**: Users can interact with smart contracts more seamlessly, without worrying about the underlying blockchain complexities. The verification model can be tailored to feel like familiar web2 experiences.
2. **Enhanced Security**: By shifting validation logic to smart contracts, a variety of security checks can be implemented to guard against unauthorized transactions. This could include multi-factor authentication, whitelisting, and more.
3. **Reliable Fee Payment**: Account abstraction can enable smart contracts to pay for gas, thereby relieving end-users from managing volatile gas prices or even paying for gas at all.

In the following sections, we'll discuss how Abstract utilizes the concept of account abstraction, ensuring modularity, security, and scalability in applications built using the Abstract SDK.

## Account Abstraction on Abstract

Within Abstract, account abstraction manifests itself in the Abstract Accounts or smart contract wallets, which are designed to offer an easy-to-use and secure interaction model for users. You can read more about their architecture in the next section.
