Pion --> XION
-
2024-05-01T07:06:32.151712Z  INFO ThreadId(01) Creating new clients, new connection, and a new channel with order ORDER_UNORDERED
2024-05-01T07:06:34.419459Z  INFO ThreadId(01) foreign_client.create{client=xion-testnet-1->pion-1:07-tendermint-0}: 🍭 client was created successfully id=07-tendermint-283
2024-05-01T07:06:40.147330Z  INFO ThreadId(01) foreign_client.create{client=pion-1->xion-testnet-1:07-tendermint-0}: 🍭 client was created successfully id=07-tendermint-124
2024-05-01T07:06:41.367369Z  INFO ThreadId(01) 🥂 pion-1 => OpenInitConnection(OpenInit { Attributes { connection_id: connection-191, client_id: 07-tendermint-283, counterparty_connection_id: None, counterparty_client_id: 07-tendermint-124 } }) at height 1-14357508
2024-05-01T07:06:58.101658Z  INFO ThreadId(01) 🥂 xion-testnet-1 => OpenTryConnection(OpenTry { Attributes { connection_id: connection-60, client_id: 07-tendermint-124, counterparty_connection_id: connection-191, counterparty_client_id: 07-tendermint-283 } }) at height 1-7657762
2024-05-01T07:07:20.805247Z  INFO ThreadId(01) 🥂 pion-1 => OpenAckConnection(OpenAck { Attributes { connection_id: connection-191, client_id: 07-tendermint-283, counterparty_connection_id: connection-60, counterparty_client_id: 07-tendermint-124 } }) at height 1-14357525
2024-05-01T07:07:31.165653Z  INFO ThreadId(01) 🥂 xion-testnet-1 => OpenConfirmConnection(OpenConfirm { Attributes { connection_id: connection-60, client_id: 07-tendermint-124, counterparty_connection_id: connection-191, counterparty_client_id: 07-tendermint-283 } }) at height 1-7657768
2024-05-01T07:07:37.373497Z  INFO ThreadId(01) connection handshake already finished for Connection { delay_period: 0ns, a_side: ConnectionSide { chain: BaseChainHandle { chain_id: pion-1 }, client_id: 07-tendermint-283, connection_id: connection-191 }, b_side: ConnectionSide { chain: BaseChainHandle { chain_id: xion-testnet-1 }, client_id: 07-tendermint-124, connection_id: connection-60 } }
2024-05-01T07:07:39.715700Z  INFO ThreadId(01) 🎊  pion-1 => OpenInitChannel(OpenInit { port_id: wasm.neutron1uhlcjl3j70zsy2r5d460g5uc5qyg42p6fpm7srwtmpw8drse9huspyv8r4, channel_id: channel-772, connection_id: None, counterparty_port_id: wasm.xion1ju236dlcrejqqhv8gy40jne94wxz6qafllsydgkhd5fzcaurm8dql58unf, counterparty_channel_id: None }) at height 1-14357534
2024-05-01T07:07:55.152778Z  INFO ThreadId(01) 🎊  xion-testnet-1 => OpenTryChannel(OpenTry { port_id: wasm.xion1ju236dlcrejqqhv8gy40jne94wxz6qafllsydgkhd5fzcaurm8dql58unf, channel_id: channel-196792, connection_id: connection-60, counterparty_port_id: wasm.neutron1uhlcjl3j70zsy2r5d460g5uc5qyg42p6fpm7srwtmpw8drse9huspyv8r4, counterparty_channel_id: channel-772 }) at height 1-7657772
2024-05-01T07:08:08.660895Z  INFO ThreadId(01) 🎊  pion-1 => OpenAckChannel(OpenAck { port_id: wasm.neutron1uhlcjl3j70zsy2r5d460g5uc5qyg42p6fpm7srwtmpw8drse9huspyv8r4, channel_id: channel-772, connection_id: connection-191, counterparty_port_id: wasm.xion1ju236dlcrejqqhv8gy40jne94wxz6qafllsydgkhd5fzcaurm8dql58unf, counterparty_channel_id: channel-196792 }) at height 1-14357547
2024-05-01T07:08:21.582776Z  INFO ThreadId(01) 🎊  xion-testnet-1 => OpenConfirmChannel(OpenConfirm { port_id: wasm.xion1ju236dlcrejqqhv8gy40jne94wxz6qafllsydgkhd5fzcaurm8dql58unf, channel_id: channel-196792, connection_id: connection-60, counterparty_port_id: wasm.neutron1uhlcjl3j70zsy2r5d460g5uc5qyg42p6fpm7srwtmpw8drse9huspyv8r4, counterparty_channel_id: channel-772 }) at height 1-7657777
2024-05-01T07:08:24.706140Z  INFO ThreadId(01) channel handshake already finished for Channel { ordering: ORDER_UNORDERED, a_side: ChannelSide { chain: BaseChainHandle { chain_id: pion-1 }, client_id: 07-tendermint-283, connection_id: connection-191, port_id: wasm.neutron1uhlcjl3j70zsy2r5d460g5uc5qyg42p6fpm7srwtmpw8drse9huspyv8r4, channel_id: channel-772, version: polytone-1 }, b_side: ChannelSide { chain: BaseChainHandle { chain_id: xion-testnet-1 }, client_id: 07-tendermint-124, connection_id: connection-60, port_id: wasm.xion1ju236dlcrejqqhv8gy40jne94wxz6qafllsydgkhd5fzcaurm8dql58unf, channel_id: channel-196792, version: polytone-1 }, connection_delay: 0ns }
SUCCESS Channel {
    ordering: Unordered,
    a_side: ChannelSide {
        chain: BaseChainHandle {
            chain_id: ChainId {
                id: "pion-1",
                version: 1,
            },
            runtime_sender: Sender { .. },
        },
        client_id: ClientId(
            "07-tendermint-283",
        ),
        connection_id: ConnectionId(
            "connection-191",
        ),
        port_id: PortId(
            "wasm.neutron1uhlcjl3j70zsy2r5d460g5uc5qyg42p6fpm7srwtmpw8drse9huspyv8r4",
        ),
        channel_id: Some(
            ChannelId(
                "channel-772",
            ),
        ),
        version: Some(
            Version(
                "polytone-1",
            ),
        ),
    },
    b_side: ChannelSide {
        chain: BaseChainHandle {
            chain_id: ChainId {
                id: "xion-testnet-1",
                version: 1,
            },
            runtime_sender: Sender { .. },
        },
        client_id: ClientId(
            "07-tendermint-124",
        ),
        connection_id: ConnectionId(
            "connection-60",
        ),
        port_id: PortId(
            "wasm.xion1ju236dlcrejqqhv8gy40jne94wxz6qafllsydgkhd5fzcaurm8dql58unf",
        ),
        channel_id: Some(
            ChannelId(
                "channel-196792",
            ),
        ),
        version: Some(
            Version(
                "polytone-1",
            ),
        ),
    },
    connection_delay: 0ns,
}


XION --> PION

2024-05-01T07:13:32.502128Z  INFO ThreadId(01) running Hermes v1.7.0+ac916f5
2024-05-01T07:13:42.870249Z  INFO ThreadId(01) 🎊  xion-testnet-1 => OpenInitChannel(OpenInit { port_id: wasm.xion150s3sev86v0lr0qn4s0k8xtzvfm42jvnc26wktky0ypsf2txeqps2gs6kn, channel_id: channel-196900, connection_id: None, counterparty_port_id: wasm.neutron1s057u4g6tc649hdw6kqmh85cjw3ekx8eyv3v9cven88zv8ewtlqqgmpt26, counterparty_channel_id: None }) at height 1-7657837
2024-05-01T07:13:52.155979Z  INFO ThreadId(01) 🎊  pion-1 => OpenTryChannel(OpenTry { port_id: wasm.neutron1s057u4g6tc649hdw6kqmh85cjw3ekx8eyv3v9cven88zv8ewtlqqgmpt26, channel_id: channel-773, connection_id: connection-191, counterparty_port_id: wasm.xion150s3sev86v0lr0qn4s0k8xtzvfm42jvnc26wktky0ypsf2txeqps2gs6kn, counterparty_channel_id: channel-196900 }) at height 1-14357694
2024-05-01T07:14:06.506880Z  INFO ThreadId(01) 🎊  xion-testnet-1 => OpenAckChannel(OpenAck { port_id: wasm.xion150s3sev86v0lr0qn4s0k8xtzvfm42jvnc26wktky0ypsf2txeqps2gs6kn, channel_id: channel-196900, connection_id: connection-60, counterparty_port_id: wasm.neutron1s057u4g6tc649hdw6kqmh85cjw3ekx8eyv3v9cven88zv8ewtlqqgmpt26, counterparty_channel_id: channel-773 }) at height 1-7657842
2024-05-01T07:14:16.228233Z  INFO ThreadId(01) 🎊  pion-1 => OpenConfirmChannel(OpenConfirm { port_id: wasm.neutron1s057u4g6tc649hdw6kqmh85cjw3ekx8eyv3v9cven88zv8ewtlqqgmpt26, channel_id: channel-773, connection_id: connection-191, counterparty_port_id: wasm.xion150s3sev86v0lr0qn4s0k8xtzvfm42jvnc26wktky0ypsf2txeqps2gs6kn, counterparty_channel_id: channel-196900 }) at height 1-14357704
2024-05-01T07:14:19.444466Z  INFO ThreadId(01) channel handshake already finished for Channel { ordering: ORDER_UNORDERED, a_side: ChannelSide { chain: BaseChainHandle { chain_id: xion-testnet-1 }, client_id: 07-tendermint-124, connection_id: connection-60, port_id: wasm.xion150s3sev86v0lr0qn4s0k8xtzvfm42jvnc26wktky0ypsf2txeqps2gs6kn, channel_id: channel-196900, version: polytone-1 }, b_side: ChannelSide { chain: BaseChainHandle { chain_id: pion-1 }, client_id: 07-tendermint-283, connection_id: connection-191, port_id: wasm.neutron1s057u4g6tc649hdw6kqmh85cjw3ekx8eyv3v9cven88zv8ewtlqqgmpt26, channel_id: channel-773, version: polytone-1 }, connection_delay: 0ns }
SUCCESS Channel {
    ordering: Unordered,
    a_side: ChannelSide {
        chain: BaseChainHandle {
            chain_id: ChainId {
                id: "xion-testnet-1",
                version: 1,
            },
            runtime_sender: Sender { .. },
        },
        client_id: ClientId(
            "07-tendermint-124",
        ),
        connection_id: ConnectionId(
            "connection-60",
        ),
        port_id: PortId(
            "wasm.xion150s3sev86v0lr0qn4s0k8xtzvfm42jvnc26wktky0ypsf2txeqps2gs6kn",
        ),
        channel_id: Some(
            ChannelId(
                "channel-196900",
            ),
        ),
        version: Some(
            Version(
                "polytone-1",
            ),
        ),
    },
    b_side: ChannelSide {
        chain: BaseChainHandle {
            chain_id: ChainId {
                id: "pion-1",
                version: 1,
            },
            runtime_sender: Sender { .. },
        },
        client_id: ClientId(
            "07-tendermint-283",
        ),
        connection_id: ConnectionId(
            "connection-191",
        ),
        port_id: PortId(
            "wasm.neutron1s057u4g6tc649hdw6kqmh85cjw3ekx8eyv3v9cven88zv8ewtlqqgmpt26",
        ),
        channel_id: Some(
            ChannelId(
                "channel-773",
            ),
        ),
        version: Some(
            Version(
                "polytone-1",
            ),
        ),
    },
    connection_delay: 0ns,
}