# Ping pong Application

The ping-pong app is an example application that demonstrates the IBC features of the abstract framework. It's has a simple function `PingPong` that accepts two parameters:

```rust
PingPong {
    /// How many pings pongs in and out should be done
    pongs: u32,
    /// Host chain
    host_chain: TruncatedChainId,
}
```

The `host_chain` is the blockchain where this app will also be deployed on. The `pongs` parameter is the number of times the app will send a ping to the host chain and wait for a pong back.
