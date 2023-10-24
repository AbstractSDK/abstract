![Abstract Nightly Build](https://github.com/AbstractSDK/abstract/actions/workflows/nightly.yml/badge.svg)

# Abstract

Monerepo for the [Abstract Money](https://abstract.money) project.

## Repository Structure

The repository is structured as follows:

- **app-template:** A template for creating new Abstract apps. See [app-template repo](https://github.com/AbstractSDK/app-template) for a cloneable version.
- **framework:** The Abstract framework. This is the core of the Abstract project and contains the smart-contracts and packages related to the on-chain abstract accounts and surrounding infrastructure. Have a look at the [Abstract SDK documentation](https://docs.abstract.money/3_framework/1_abstract_sdk.html) if you're interested in building with Abstract!
- **modules:** The Abstract-maintained modules. These are a set of example apps that are built on top of the Abstract framework. They can be used as standalone applications or as building blocks for creating more complex applications.
- **integrations:** Integrations with other projects, particularly dexes.
- **scripts:** Scripts for CI.
- **interchain:** Contains a testing suite for Abstract Interchain Accounts as well as scripts to deploy the framework to the interchain.