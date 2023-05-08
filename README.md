# Abstract App Module Template

Apps are composable smart-contracts that enable features or transform Accounts into standalone products.
Every app is instantiated for each OS individually and is migratable.
Apps are allowed to perform actions on the proxy contract.

The might also want to integrate with other enabled modules/apis, and have access to the ans_host contract for address
storage.

Therefore the the base state includes

- Proxy address
- Manager address
- AnsHost address (configurable)

The proxy and manager addresses are not configurable after instantiation.

## Setup
```shell
chmod +x ./template-setup.sh
./template-setup.sh
```
