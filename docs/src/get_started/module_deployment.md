# Module Deployment

Deploying your module is an easy 3-step process: Module Uploading, Registration and Schema Linking. Let's go over each step in detail.

> This doc assumes you're using the module app template, if you're not we recommend looking at the relevant files in the template to set up your own deployment process.

## Module Uploading

Uploading your module involves first compiling your module as a WASM binary and then uploading it to the network(s) you want your module to be available on. This will yield you a code_id that is a unique identifier for your module on the network.

### Compiling your module

You can compile your module by running the following command:

```bash
$ just wasm
> Compiling to WASM...
```

> The WASM optimizer uses a docker container to compile your module. If you don't have docker installed you can install it from [here](https://docs.docker.com/get-docker/).

This should result in an `artifacts` directory being created in your project root. Inside you will find a `my_module.wasm` file that is your module's binary.

Now you can go ahead and deploy the module to the network(s) you want to make it available on. You can do this by running the following command:

<!-- TODO: re-add cli to deploy command in template or create Abstract CLI -->

```bash
$ just deploy uni-1
> Deploying module...
```

This will use the module's `examples/deploy.rs` script to deploy the module to the `uni-1` network. The resulting code-id of your contract should now be in the `state.json` file created for you.

### Module Registration

Once uploaded you can register the module on the Abstract Version Control using our web-app. To do this, go to [the registration page](TODO: insert link) and use the module's code-id to register it. **The module_id you assign to the module should be identical to the module_id that you used in the module builder.** If not module installation will fail.

### Module Installation

## JSON Schema Linking

To improve the user-experience for developers using your module we recommend linking your module's JSON schema to the Abstract Version Control. This will allow developers to use the Abstract web app to interact with your module.

TODO: Add steps to link schema
1. Generate schema by running `just schema`
2. PR schema to [Abstract Schema Repository](https://github.com/AbstractSDK/schemas). Once approved the front-end will allow for message generation for your module.