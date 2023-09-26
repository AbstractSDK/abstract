# Monetization

In the Abstract framework, developers have the capability to monetize their modules by setting an installation fee for
others to use their modules. By introducing monetization strategies, Abstract offers developers incentives to build and
share valuable modules with the community.

Here's a concise breakdown of how this works:

- Modules can be installed into abstract accounts.
- Each module can be configured with a **Monetization strategy**, primarily:
    - **InstallFee**: A fee set by the developer which must be paid by other users to install the module.
      This fee is then transferred to the namespace owner's account, which is fetched from the version control registry.
    - **None**: No monetization strategy is applied for the module.

All module monetization details are stored in the version control but are verified and enforced by the module factory.

To assist users in budgeting, the module factory provides the `SimulateInstallModules` query, which returns the total
sum of funds required to install a specified set of modules, including monetization and initialization funds.
