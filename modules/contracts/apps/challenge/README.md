# Challenge App

## Description

The Challenge App Module is used to create challenges with friends and motivate completing challenges by striking admin with chosen asset in case challenge got failed.

## Why use the Challenge App?

The Challenge App Module offers a creative solution to the common problem of losing motivation and failing to achieve personal or group goals. As a tool for creating challenges with friends and incorporating a stake of a chosen asset that is forfeited if the challenge is failed, brings an innovative approach to motivation and accountability among peers.

## Features
- Admin of this contract can create challenge 
  - Name and description of the challenge
  - List of friends(either by address or Abstract Account id) that have a voting power for failing a challenge
  - Asset for striking
  - Striking strategy:
    - Amount per friend (per_friend)
    - Split amount between friends (split)
  - Challenge duration
  - Proposal duration
  - Strikes limit (max amount of times admin getting striked for failing this challenge)
- Friends can vote on challenges
  - When friend votes on a challenge new proposal will get created
  - During proposal period other friends can vote
  - After proposal period (and veto period if configured) anyone can execute `count_votes` to count votes and in case votes for punish passed threshold - strike an admin
- During veto period admin can veto this vote
- Between proposals admin can edit list of friends for this challenge

## Installation

To use the Challenge App Module in your Rust project, add the following dependency to your `Cargo.toml`:
```toml
challenge-app = { git = "https://github.com/AbstractSDK/abstract.git", tag = "v0.19.0", default-features = false }
```

## Documentation

- **App Documentation**: Comprehensive information about adapters can be found in the [official documentation](https://docs.abstract.money/framework/module_types.html#apps).

## Contributing

If you have suggestions, improvements or want to contribute to the project, we welcome your input on [GitHub](https://github.com/AbstractSDK/abstract).

## Community
Check out the following places for support, discussions & feedback:

- Join our [Discord server](https://discord.com/invite/uch3Tq3aym)
