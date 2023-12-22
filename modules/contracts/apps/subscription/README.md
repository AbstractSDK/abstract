# Subscription App

## Description

This app allows users to create subscriptions that other people can subscribe to. Users provide funds when they subscribe and those funds are used to pay for the subscription for as long as the funds don't run out.

When a user does'n top-up their balance, an external call is made to the `subscription` contract to cancel the subscription. The admin can opt to add a cancellation hook that will be called when the subscription is canceled and which contains the addresses of the now ex-subscribers.

## TODO: Features
