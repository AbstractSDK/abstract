# Subscription App

The subscription app is an Abstract App that allows users to create configurable subscriptions for their services. Be it on, or off-chain.

## Description

The subscription app allows users to create subscriptions that other people can subscribe to. Subscribers must provide funds when they subscribe, and those funds are used to pay for the subscription for a certain amount of time. When the subscription is about to expire, the subscriber must top-up their balance to continue using the service.

When a subscriber does'n top-up their balance, an external call is made to the subscription app to cancel the subscription. The creator of the app can opt to add a cancellation hook that will be called when the subscription is canceled and which contains the addresses of the users that need to be un-subscribed.

By configuring this hook any developer can create an on-chain service that only works for users that have a valid subscription.

## Features

- Create customizable subscriptions in terms of coins/second. For example, you can configure the contract to accept 1 uatom per second. This would come down to a yearly cost of ~31.5 ATOM/year.

- Unsubscribe hook: Let your other contracts know when a user stopped paying for their subscription.

- Income tracking: Keep track of how much income the subscription has generated within a customizable time frame. This can be use to control emissions of tokens or simply to keep track of how much money the subscription has generated.

- Easily queryable off-chain: With cw-orchestrator you can easily query the subscription app to figure out if a user has a valid subscription. You can then create off-chain services that only work for users that have a valid subscription.

If you have a specific service that you'd like to start offering subscriptions for, please reach out to us on <a href="https://discord.gg/uch3Tq3aym" target="_blank">Discord</a> and we'll help you get started!
