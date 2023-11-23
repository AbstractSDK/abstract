# Challenge App

The Challenge App Module is used to create challenges with friends and motivate completing challenges by striking admin with chosen asset in case challenge got failed.

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