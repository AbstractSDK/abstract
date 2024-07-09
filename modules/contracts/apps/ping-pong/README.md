# Ping pong Application

The ping-pong app is an example application that demonstrates the IBC features of the abstract framework. It's has a simple function `PingPong` that accepts a counterparty chain to ping-pong against. 

```rust
PingPong {
    /// Truncated chain-id of the opponent chain.
    opponent_chain: TruncatedChainId,
}
```

The `opponent_chain` is the blockchain where the app is also deployed. 

## Ping-pong Flow

When called, the app will send an IBC packet to an instance of the ping-pong app on the opponent chain. On receiving that packet (`src/ibc/module.rs`) the app will assert if the chain's height is an even number. If the block height is even, the app will respond to the message with a "pong" attribute in the `Response`. If it is uneven, the app will save a loss.

In both cases (if the opponent responds with "pong" or loses) a callback will be called on the contract (`src/ibc/callback.rs`). On receiving this callback the execution result will be parsed and checked for the presence of a "pong" attribute. If it is present that means the game goes on and the contract will check its own block height and assert wether it plays or loses. 

## IBC Queries

An altenative method `QueryAndMaybePingPong` uses IBC queries to figure out if it should play against the opponent or not. In this hypothetical example the local contract queries the ping-pong app on the remote chain on the `QueryMsg::BlockHeight` query which returns the block height of the chain. 

The callback received in `src/ibc/callback.rs` will then contain the response to that query. If the block height is uneven, that indicates that playing a game should result in a win, thus the app sends a "ping" message. 

> Blocks get produced continuously so the odds of winning won't really be effected. 
