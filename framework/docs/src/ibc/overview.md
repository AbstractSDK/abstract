# Interchain Abstract Accounts

Interchain Abstract Accounts is Abstract's solution to chain-agnostic accounts. It allows users to create an account on
one chain and use it on any other chain that supports Abstract. This is achieved by using a combination of
the <a href="https://ibcprotocol.org/" target="_blank">Inter-Blockchain Communication (IBC) protocol</a> and
the [Abstract Accounts](../3_framework/3_architecture.md).

## Overview

IAA allow users to interact with any smart-contract on any chain using their local account. This mechanism is powered by a set of Abstract smart-contracts that will dispatch messages that users send locally to a remote chain.

### Account creation

The first step of using Interchain Abstract Account is creating a remote account.
