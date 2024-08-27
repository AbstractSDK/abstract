# Account Ownership

Abstract Accounts can be owned by any (custom) governance infrastructure. Because most developers appreciate an easy-to-use interface to control their Account, Abstract supports two fully integrated governance structures that ensure a seamless user experience.

When configuring the governance for your Account, you will be prompted to choose between supported governance types, *Monarchy* or *Multi-signature*.

```admonish info
Not interested in account ownership? Skip to our section on [Framework Components](../platform/index.md).
```

## Monarchy

In a monarchy, a single wallet has full control over the Account. If you're connected with a wallet, your address will be automatically inserted as the owner.

```mermaid
graph TD
    A[Single Account] -->|Controls| B(Abstract Account)
```

## Multi-signature

Multi-signature ("multisig") governance is a governance structure that requires a subset of its members to approve an action before it can be executed. Abstract implemented this functionality with the <a href="https://github.com/CosmWasm/cw-plus/blob/main/packages/cw3/README.md" target="_blank"> cw-3 standard</a>.

Here are a few terms you need to know about when configuring your multisig:

- *Voter weight* 🏋️‍♂️: The weight that the voter has when voting on a proposal.
- *Threshold* 📊: The minimal % of the total weight that needs to vote YES on a proposal for it to pass.

```mermaid
graph TD
    subgraph Voters
        V1[Voter 1]
        V2[Voter 2]
        V3[Voter 3]
    end

    V1 --> A[Multisig Wallet]
    V2 --> A
    V3 --> A
    
    A -->|Controls| B(Abstract Account)

    B[Abstract Account]
```

### Example

Suppose you share an account with your friends and want to use a multisig governance structure to prevent unilateral control over the account. You have five stakeholders, and you want at least 60% of the total voting weight to approve a proposal for it to pass.

1. Set up the multisig module in your dApp.
2. Assign voter weights to each of the five stakeholders. For instance, A: 30%, B: 20%, C: 20%, D: 15%, and E: 15%.
3. Configure the multisig module with a 60% threshold.

With this configuration, any proposal will require approval from stakeholders with a combined voting weight of at least 60% to be executed. This ensures a more democratic decision-making process and reduces the risk of a single stakeholder making unilateral decisions.

## Sub-Accounts

A Sub-Account is an Abstract Account that is owned by another Abstract Account. They are important to users as they allow users to safely experiment with different apps without the concern of those apps accessing funds from their main account or other apps.

Sub-accounts are easily created by calling `CreateSubAccount` on any account. The diagram below shows how sub-accounts are owned by a main `Account` or other sub-accounts.

```mermaid
flowchart TB
    Account
    SubAccount-A
    SubAccount-B
    SubAccount-C
    Owner --> Account
    Account --> SubAccount-A
    Account --> SubAccount-B
    SubAccount-A --> SubAccount-C
```

To simplify accessing or configuring a sub-account or app we allow calling any sub-account or any app on a sub-account directly without requiring the message to be proxied through the top-level account. The diagram below shows how an account owner can configure the sub-accounts and apps directly that are part of his main account.

```mermaid
flowchart TB
    direction TB
    subgraph AbstrA[Sub-Account A]
        direction TB
        ManagerA[Manager] --> ProxyA[Proxy]
        AppA[App]
    end

    subgraph AbstrB[Sub-Account B]
        direction TB
        ManagerB[Manager] --> ProxyB[Proxy]
    end

    subgraph AbstrC[Sub-Account C]
        direction TB
        ManagerC[Manager] --> ProxyC[Proxy]
        App
    end

    subgraph Abstr[Account]
        direction TB
        Manager --> Proxy
    end

Owner --> Manager
Manager --> ManagerA
Manager ---> ManagerB
ManagerB --> ManagerC

Owner -.Configure App.....-> AppA
Owner -.Configure Account....-> ManagerC
```

As a result of this structure, complex multi-account systems can easily be transferred between governance systems by simply changing the owner of the top-level account.

```admonish info
Sub-accounts have a depth of 2. I.e. an account can have sub-accounts, and those sub-accounts can have sub-accounts themselves, but no further.
```
