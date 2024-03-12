# Subscription App

## Description

This app allows users to create subscriptions that other people can subscribe to. Users provide funds when they subscribe and those funds are used to pay for the subscription for as long as the funds don't run out.

When a user does'n top-up their balance, an external call is made to the `subscription` contract to cancel the subscription. The admin can opt to add a cancellation hook that will be called when the subscription is canceled and which contains the addresses of the now ex-subscribers.

## Why use the Subscription App?

TBD

## Features
TBD

## Installation
TBD

## Documentation

- **App Documentation**: Comprehensive information about adapters can be found in the [official documentation](https://docs.abstract.money/3_framework/6_module_types.html#apps).

## Contributing

If you have suggestions, improvements or want to contribute to the project, we welcome your input on [GitHub](https://github.com/AbstractSDK/abstract).
