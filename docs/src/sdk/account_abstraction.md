# Account Abstraction
Account abstraction is a new core concept within blockchain and smart contract platforms, designed to streamline how users interact with decentralized applications (dApps). Its fundamental idea is to abstract away the complexities of blockchain interactions and provide a user-friendly, secure interface for transactions.

In traditional blockchain models, the transaction initiation process is typically rigid, where users directly sign and send transactions. Account abstraction simplifies this process by making the transaction initiation flexible and programmable. Essentially, it allows the transaction logic to be customized within smart contracts, vastly extending the capabilities of applications leveraging the abstraction.

Also see [EIP-4337](https://eips.ethereum.org/EIPS/eip-4337) to read about account abstraction in the Ethereum ecosystem.

This concept of account abstraction, when implemented correctly, can provide numerous benefits:

1.  **Improved User Experience**: Users can interact with smart contracts more seamlessly, without worrying about the underlying blockchain complexities. The interaction model can be tailored to align with familiar web2 paradigms.
2.  **Enhanced Security**: By shifting transaction logic to smart contracts, a variety of security checks can be implemented to guard against unauthorized transactions.
3.  **Flexibility**: It allows for innovative transaction types and enables advanced features such as transaction batching, atomic swaps, and more.
4.  **Reliable Fee Payment**: Account abstraction can enable smart contracts to pay for gas, thereby relieving end-users from managing volatile gas prices or even paying for gas at all.

In the following sections, we'll discuss how Abstract utilizes the concept of account abstraction, ensuring modularity, security, and scalability in applications built using the Abstract SDK.

## Account Abstraction on Abstract


> TODO: deepdive in the architecture instead of giving an overview here 

Within Abstract, account abstraction is a foundational component. Here, it manifests as abstracted accounts or smart contract wallets, which are designed to offer an easy-to-use and secure interaction model for users. These are known as Abstract Accounts.

Read [Abstract Account Architecture](./architecture) for a deep dive on the implementation of account abstraction in Abstract.
