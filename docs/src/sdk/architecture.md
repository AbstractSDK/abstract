# Abstract Architecture

The Manager module is responsible for managing permissions and executing contract calls on behalf of other modules within the application. It serves as the entry point for external calls, and provides a secure and controlled environment for managing the execution of smart contracts. The Manager module can also be used to manage and coordinate the interactions between different modules within an application.

The Proxy module, on the other hand, is responsible for holding funds and interacting with external contracts. It serves as the exit point for the application, and is responsible for managing the flow of funds in and out of the application. The Proxy module is also responsible for authorizing the use of funds in smart contract transactions, and for ensuring the security and integrity of the application's funds.

The Manager and Proxy modules work together to provide a secure and controlled environment for managing the execution of smart contracts, managing the flow of funds in and out of the application, and ensuring the security and integrity of the application's funds. The Manager module manages the permissions and execution of smart contracts within the application, while the Proxy module manages the flow of funds in and out of the application and authorizes the use of funds in smart contract transactions. Together, the Manager and Proxy modules provide a foundation for building decentralized applications on the blockchain.

```mermaid
    graph TD;
        A-->B;
        A-->C;
        B-->D;
        C-->D;
```
