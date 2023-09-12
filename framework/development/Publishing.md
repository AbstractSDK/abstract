# Publish guide

Publishing new abstract versions.

1. Bump the abstract version everywhere it is used.

Now you should proceed with deploying the contracts to the different chains. We want to deploy to as many chains as possible because a new version of abstract-interfaces will have to be published when new chains are added.

After deploying the contracts you can proceed with publishing the rest of the contracts. Before doing this ensure that:

- You have the latest x86_64 artifacts in `./framework/artifacts`. These should be automatically committed by the CI.
<!-- - You have updated the version abstract-interfaces for all its dependencies. -->
- You have all the latest addresses and code-ids in the `state.json` file located in the `abstract-interface` folder.

Releasing is done in the following order:

1. `abstract_core`
2. `abstract-testing`
3. `abstract-sdk`
4. All contracts in `./contracts`

Do this by running `./publish/contracts.sh`.

Now we have all the contracts published.

> Do *NOT* use the ARM-generated artifacts when deploying or publishing. They are only used for testing.

Publishing is then performed by running `./publish/packages.sh`.

The resulting data (addresses, code-ids) is used when publishing the abstract-interface.
5. `abstract-interface`
6. `abstract-adapter`, `abstract-app` and `abstract-ibc-host`
