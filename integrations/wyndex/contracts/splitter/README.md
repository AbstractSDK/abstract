# Splitter contract

During instantiation, you provide list of addresses along with weights do divide future payments. Weights must sum up to 1.0.
If you want to split cw20 tokens, you need to specify all necessary addresses in InstantiateMsg.

While calling execute msg, contract's balance of native tokens will get splitted and sent further down according to the configuration.
