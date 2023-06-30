# Publish guide

Publishing new abstract versions.

## Utils

Two utils packages (`abstract-ica` and `abstract-macros`) are used by a lot of packages and are considered stable. They are published manually.

## Packages

Ideally only update the packages that have changed. If you are unsure, update all packages.  
By changing the version in the Cargo workspace you change the version of all contracts and `abstract-interface`.
`abstract-interface` is a wrapper package around all the abstract contracts and is used extensively in testing.

New releases of `abstract_core`, `abstract-sdk` or `abstract-testing` should be reflected in the Cargo workspace
file.

1. `abstract_core`
2. `abstract-testing`
3. `abstract-sdk`
4. All contracts in `./contracts`

Now you should proceed with deploying the contracts to the different chains. The resulting data (addresses, code-ids) is used when publishing the abstract-interface.
5. `abstract-interface`
6. `abstract-adapter`, `abstract-app` and `abstract-ibc-host`
