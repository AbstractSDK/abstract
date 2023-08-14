# Abstract Gas Token Issuer
The goal of this module is to allow your Account to issue tokens representing a fee-grant from the Account. This can be used not only to create a new Cosmos-SDK EOA, but also for allowing the user to use gas on your Account's behalf. You essentially can give them an onboarding token.

## Details
The module is designed to be as simple as possible. After installing the module, your Account must give AuthZ permissions to this module to FeeGrant on its behalf. Then, the Account can issue tokens to any address it wants. The tokens are not transferable by default, but they could be made transferable if you want to allow the user to sell them on the open market.

The benefit of tokenizing the gas is that you are essentially issuing a gas gift card, so if the gas is not spent by the user then nothing is lost by you. This is a great way to onboard users to your application. You can also limit the amount of gas that can be spent by the user, so you can control your risk, as well as the specific calls that can be made.

1. Install Gas Token Issuer Module
2. Give AuthZ permissions to the module to FeeGrant on your Account's behalf
3. Issue tokens to any address you want
4. The user can spend the gas on your Account's behalf

The reason that this is better than plain ol' FeeGrant is that it's more trackable and easier to manage.
