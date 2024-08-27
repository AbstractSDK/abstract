# Account Oracle

The Account Oracle is an integrated on-chain service that allows you to retrieve the value of all assets held in an account in terms of a base asset. This simple functionality can be used to create account-based debt positions, automated trading bots and much more!

```admonish info
This section is mainly focused on developers. If you're a user feel free to skip this section!
```

## Value is relative

The value of something is always relative to some other thing. A bitcoin isn't valued at 20,000, it's valued at 20,000 **USD**. Likewise the first setting required to configure Abstract's oracle is: In what currency do you want your Account's assets to be valued?

We'll call this currency the *base asset*. There can never be more than one base asset and every asset will be valued in terms of this base asset.

With a base asset selected you can set up a *value-reference* for each asset that is held in your Account. A value-reference is a configuration that references some data that tells the contract how it should determine the value for that asset. Consequently your base asset won't have an associated value-reference as everything is valued relative to it! Don't worry, we'll show some examples after covering the basics.

## Types of Value-References

To ensure that you configure and use the oracle correctly you'll need to understand, on a high level, how the value-reference system works and what its limitations are. Your app's security might depend on it!

```admonish hint
Remember: Every asset, apart from the base asset, has an associated value-reference and that value-reference allows the Oracle to determine the value of the asset in terms of the base asset.
```

Currently there are four value-reference types that can be applied to an asset. Lets go over them.

### **1. Reference Pool**

This is the most common value-reference type. It points to a dex trading pair where one of the two assets of the pair is the asset on which this value-reference is applied. The Account takes this information and does three things.

1. Query how much X the Account holds.
2. Determine the price of asset X, defined by the pool X/Y.
3. Calculate the value of asset X in terms of asset Y given the price.
This gives us the value of asset X in terms of asset Y.

```admonish example
Your Account has 10 $JUNO and 50 $USDC. You'd like to be shown the value of your assets in terms of USD.
1. You identify that you want every asset denominated in US dollars. Therefore you choose $USDC as your base asset.
2. You identify the easiest route to swap your $JUNO for $USDC which is a trading pair on Osmosis. Therefore you add $JUNO to your Account with a Pool value-reference.
3. The ratio of $JUNO/$USDC in the pool is 1/10 so 1 $JUNO = 10 $USDC.

The Oracle can then presume that if you would swap your $JUNO to $USDC in that pool, you would end up getting 10 $USDC. Therefore the total value of your assets is 60 $USDC.
```

### **2. Liquidity Token**

A liquidity token is nothing more than a claim on a set of assets in a liquidity pool. Therefore the value of each liquidity token is defined by the composition of asset held within that pool.

### **3. Value As**

You might want to set the value of some asset in terms of another asset. For example, you could argue that every stablecoin is equal in value, irrespective of small fluctuations between them.

### **4. External**

Some assets/positions are more complex. These include, but are not limited to: staked tokens, locked tokens and most third-party vault-like products. The Account needs to interact with Adapter modules that interact with these services to find out how it should value the asset/position.

## Use With Caution

As we've outlined, each asset is valued relative to an asset or multiple assets. By recursively calling this value-reference function on each asset we can determine the value of any asset relative to our base asset.

```admonish warning
As each asset's value is referenced to some other asset through a price relation it exposes assets with a weak link to the base asset to a lot more volatility and attack surface. Therefore we recommend that you select a highly-liquid base asset with the highest liquidity trading pairs when configuring your assets.
```

```admonish danger
While this way of determining an asset's value is very intuitive, it doesn't account for bad actors. Manipulation of asset prices to trigger smart-contract actions isn't uncommon. Therefore we don't recommend this version of the Account for creating high-value automated-trading products. No worries, an implementation based on oracle prices is in the works!
```

Any questions regarding the oracle and its configuration can be asked in our [Discord](https://discord.com/invite/uch3Tq3aym) server!
