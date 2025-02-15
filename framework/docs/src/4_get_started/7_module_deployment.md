# Module Deployment

Deploying your module is an easy 3-step process: Module Uploading, Registration and Schema Linking. Let's go over each
step in detail.

> This doc assumes you're using the module app template, if you're not we recommend looking at the relevant files in the
> template to set up your own deployment process.

## Module Uploading

Uploading your module involves first compiling your module as a WASM binary and then uploading it to the network(s) you
want your module to be available on. This will yield you a code_id that is a unique identifier for your module on the
network.

### Compiling your module

Once you have confirmed that your module works as expected you can spin up a local node and deploy Abstract + your app onto the chain. You need [Docker](https://www.docker.com/) installed for this step. 

You can compile your module by running the following command:

```bash
$ just wasm
> Compiling to WASM...
```

> The WASM optimizer uses a docker container to compile your module. If you don't have docker installed you can install
> it from <a href="https://docs.docker.com/get-docker/" target="_blank">here</a>.

This should result in an `artifacts` directory being created in your project root. Inside you will find
a `my_module.wasm` file that is your module's binary. 

### Publish your module

Before attempting to publish your app you need to add your mnemonic to the `.env` file. **Don't use a mnemonic that has mainnet funds for this.** Make sure this account has funds. If you don't have the deployment will fail. Get funds from respective chain faucets or ask for some test tokens on Abstract Discord.

Now you can go ahead and publish the module to the network(s) you want to make it available on. You can do this by running the following command:

```bash
$ just publish uni-6
> Deploying module...
```

This will use the module's `examples/publish.rs` script to deploy the module to the `uni-1` network. The resulting
code-id of your contract should now be in the `state.json` file created for you. The script will also attempt to
register the module on the Abstract Registry, hence the mnemonic used in the script should be the same as the one
you used to create the account and register the namespace.

## JSON Schema Linking

To improve the user-experience for developers using your module we recommend linking your module's JSON schema to the Abstract Registry. This will allow developers (and you) to use the Abstract web app to interact with your module.

```admonish warning
You need to install [github cli](https://cli.github.com/) for this step.

Follow [these install instructions](https://github.com/cli/cli#installation) as per your operating system needs.
```

To link your module's schema you can run the following command:

```bash
$ just publish-schemas <namespace> <name> <version>
> Publishing schemas...
```

Where you fill the `<namespace>`, `<name>` and `<version>` with the same values you used to register your module on the
Abstract Registry.

## Module Installation

To install your module, go to the <a href="https://console.abstract.money" target="_blank">Abstract Account Dashboard</a>, enter the dev-mode by clicking Enter Dev Mode on Action tab, go to your Account (or a new one) and click on the `Modules` tab. Here you will find a list of all the modules you have
registered on the Abstract Registry. Click on the `Install` button next to your module and select the network you
want to install it on. This will open a modal with the following fields:
