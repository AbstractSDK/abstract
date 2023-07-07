# DCA App Module

The DCA (Dollar Cost Averaging) app module is designed to give users an automated investment tool that allows them to average into assets over time. By specifying a source asset and a target asset, users can configure the system to periodically convert a specified amount of the source asset into the target asset, effectively implementing a Dollar Cost Averaging strategy.

# Features
- This module interacts with [croncat module](../croncat/README.md) to automated schedule and with [dex adapter](../../../adapters/contracts/dex/README.md) for swaps
- Create DCA strategy
- Update DCA strategy
- Cancel DCA strategy