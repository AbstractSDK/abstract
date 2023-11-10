# Full-Stack Development

## Clone the Template
To get started, clone the Abstract application template.,

```sh
git clone https://github.com/AbstractSdk/app-template my-app
```

Check out the README for further instructions on integrating with the template.

### Process
1. Update your contracts
2. Generate the typescript SDK for the contracts
	1. This uses `ts-codegen` under the hood

### Deployment
1. Bump the version of your contracts. 
	1. Patch versions for logic changes.
	2. Minor versions for API changes..
	3. Major versions for fully-breaking changes.
2. Publish the schemas for your contracts.
	1. Run `just publish-schemas <namespace> <name> <version>` which will create a PR on the [Abstract App Schemas](https://github.com/AbstractSdk/schemas) repository, and allow for auto-generated interfaces on the frontend.
3. 