# Monetization

Our app-store provides developers with the ability to monetize their modules by configuring an installation fee for their modules. By introducing monetization strategies, Abstract offers developers incentives to build and share valuable modules with the community.

Here's a concise breakdown of how this works:

- As explained, modules are the building blocks of Abstract Apps and can be installed on Abstract Accounts.
- Modules can be developed and published to the Abstract App Store by any developer.
- Each module can be configured with a **Monetization strategy**, primarily:
  - **InstallFee**: A fee set by the developer which must be paid by other users to install the module. This fee is then transferred to the namespace owner's account, which is fetched from the version control registry.
  - **None**: No monetization strategy is applied for the module.

All module monetization details are stored in the version control but are verified and enforced by the module factory.

To assist users in budgeting, the module factory provides the `SimulateInstallModules` query, which returns the total
sum of funds required to install a specified set of modules, including monetization and initialization funds.

## Subscriptions

In addition to one-time installation fees, the Abstract framework empowers developers to introduce subscription-based monetization strategies for their modules. This model facilitates a steady stream of revenue, enhancing the sustainability and continuous development of the modules.

Subscriptions are being worked on and will be available soon, stay tuned!.
