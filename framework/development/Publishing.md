# Publish guide

Publishing new abstract versions.

1. Bump the abstract version everywhere it is used.
2. Remove the path deps from `adapters`, `adapter-packages` and `integration-bundles`. This will ensure that git-based imports use the published abstract version.

Releasing is done in the following order:

1. `abstract_core`
2. `abstract-testing`
3. `abstract-sdk`
4. All contracts in `./contracts`

Do this by running `./publish/contracts.sh`.

Now we have all the 

Now you should proceed with deploying the contracts to the different chains. The resulting data (addresses, code-ids) is used when publishing the abstract-interface.
5. `abstract-interface`
6. `abstract-adapter`, `abstract-app` and `abstract-ibc-host`
