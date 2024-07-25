# Carrot Savings

Carrot Savings is an interchain stablecoin yield aggregator and optimizer, designed to give users the best stablecoin yields using DeFi protocols.

```admonish info
[Carrot Savings is live](https://carrotsavings.com)! Deposit and withdraw USDC and get over 20% yield (as of July 15, 2024).
```

### Modules

- [Dex Adapter](../modules/dex.md)
- [Lending Market Adapter](../modules/lending-market.md)
- Savings: stablecoin position management

## Architecture

```mermaid
flowchart LR
	subgraph App[Carrot Savings]
		direction LR
		B[Savings] -.-> |provide_liquidity|Dex[/Dex/]
		B -.-> |lend / repay|Lending[/Lending/]
		Dex --> AA["Abstract Account"]
		B --> AA
		Lending --> AA
		end

	User[fa:fa-user User] -.-> |deposit|App
	User -.-> |withdraw|App
	Cron -.-> |autocompound|App
```
