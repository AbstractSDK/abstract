# CW-Placeholder

A simple placeholder contract which can be upgraded later through migration. With this you can deploy a contract to get a fixed address before deploying the real contract to simplify some processes such as Instantiation through governance proposals. 

The contract has an empty struct for Instantiate and no Execute or Query messages.
Only state it writes is the contract name and version per CW2.
When migrating from a `Placeholder` contract to your desired final contract type, the admin is passed from the `Placeholder` contract.
The end result is the same as deploying your desired contract but it retains the address gotten when this placeholder was instantiated.
