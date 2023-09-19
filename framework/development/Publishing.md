# Publish guide

Publishing new abstract versions.

1. Bump the abstract versions of packages everywhere they are used.

Now you should proceed with deploying the contracts to the different chains. We want to deploy to as many chains as possible because a new version of abstract-interfaces will have to be published when new chains are added.

After deploying the contracts you can proceed with publishing the rest of the contracts. Before doing this ensure that:

- You have the latest x86_64 artifacts in `./framework/artifacts`. These should be automatically committed by the CI before deployment is triggered.

- You have all the latest addresses and code-ids in the `state.json` file located in the `abstract-interface` folder.

You can then trigger the release by executing `./publish/contracts.sh`.

