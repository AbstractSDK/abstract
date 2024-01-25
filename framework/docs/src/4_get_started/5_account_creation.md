# Becoming a Publisher

A publisher is an entity that publishes modules to the Abstract platform. Publishers can be individuals, teams, or AI robots. The only requirement is that they have an Abstract Account with a Namespace.

But how? I hear you whisper to yourself. Well, we've made it really easy for you. All you need to do is follow the instructions below.

## Creating an Abstract Account

The first step to becoming a publisher is creating an Abstract Account.

You can either do this through our [web-app](https://console.abstract.money) or you can use the script that we've conveniently provided for you.

The script we'll outline here is provided in the app-template `deploy.rs` file. You can find it [here](https://github.com/AbstractSDK/abstract/blob/main/app-template/examples/publish.rs)

A module namespace is your (or your team's) publishing domain for Abstract modules. Through this design you can monetize your product through a namespace or on a per-modules basis as explained in more detail in the [monetization](../5_platform/6_monetization.md) section.

### The `abstract-client` Package

To easily test and publish modules programmatically, we've created the `abstract-client` package. This package is a client library that allows you to interact with the Abstract platform.

We highly recommend you use this package to interact with the Abstract platform. It will make your life a lot easier.

You can read more about the `abstract-client` package [here](https://crates.io/crates/abstract-client), or proceed with reading as we'll show you some usage of this package in the following sections.
