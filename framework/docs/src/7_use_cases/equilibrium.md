# Equilibrium Rebalancer

Equilibrium, 1st place winner of HackWasm 2022, is a rebalancing protocol for DeFi portfolios. It allows users to create weighted baskets of assets and supports rebalancing of the portfolio using liquidity on any Dex in the interchain ecosystem, local or remote.


### Modules

- [Dex Adapter](../modules/dex.md)
- ETF: tokenizes the account and allows users to buy and sell shares
- Balancer: handles basket weights and rebalancing logic


## Architecture

```mermaid
flowchart LR
	subgraph App[Equilibrium]
		direction LR
		M[Balancer] -.-> |swaps|Dex
		M -.-> |updates shares|ETF
		Dex[/Dex/] --> AA["Abstract Account"]
		M --> AA
		ETF -- tokenizes --- AA
		end

	User[fa:fa-user User] -.-> |rebalance|App
	User -.-> |deposit|App
	User -.-> |withdraw|App
	User -.-> |update_weights|App
```


Read more on equilbrium on its <a href="https://equilibrium.zone/" target="_blank">official
website</a>.
