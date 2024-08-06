# Abstract IBC Client

The IBC client contract provides a way for Abstract Accounts to create Interchain Abstract Accounts (ICAA) on other chains. It relies on <a href="https://doc.rust-lang.org/book/" target="_blank"> Polytone</a> for IBC message forwarding.

Users can enable IBC by calling the `ManagerExecuteMsg::UpdateSettings { enable_ibc: true }` on their account's manager. This will register the `abstract:ibc_client` with the account so it can be referenced by other applications.

You can learn more about Abstract IBC [in our docs](https://docs.abstract.money/3_framework/8_ibc.html)!

```mermaid
flowchart 
    subgraph Abstract Chain Abstraction
        direction LR
        subgraph CosmWasm Chain
            A4[Abstract Account]
            IH[IBC Host]
            PV[Polytone Voice]
            PP[Polytone Proxy]
        end

        subgraph Cosmos Chain
            A5[Interchain Account]
        end

        subgraph Ethereum Chain
            A6[Proxy]
            EV[EVM Voice]
            EV -.-> A6
        end

        subgraph CosmWasm Chain
        direction TB
        A3 -.-> IC[IBC Client]
        IC -.-> PN[Polytone Note]
        IC -.-> EN[EVM Note]
        IC -.-> CI[CW ICA Controller]
        end

        User2[fa:fa-user User] -- owns --> A3["Abstract Account"]
        CI -.-> A5
        PN -.-> PV
        PV -.-> PP
        PP -.-> IH
        IH -.-> A4
        EN -.-> EV
    end
```
