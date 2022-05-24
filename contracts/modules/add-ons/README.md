# Add-ons

Add-ons are feature-adding modules. They allow users to add some extra functionality to their os. 

Every add-on is migratable. Custom add-ons are allowed. 

Add-ons are allowed to perform actions on the proxy contract. 
The might also want to integrate with other enabled modules/apis, and have access to the memory contract for address storage.

Therefore the the base state includes

- Proxy address
- Manager address
- Memory address (configurable)

The proxy and manager addresses are not configurable after instantiation. We don't store the os-id here as we can enable one manager to have multiple proxy's under it in the future. 
