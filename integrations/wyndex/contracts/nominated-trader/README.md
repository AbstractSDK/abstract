# Wynd Nominated Trader 

The Nominated Trader accumulates Wyndex protocol fees and swaps them to WYND through a nominated route of pool hops. 

## Example Use

A number of pairs are created using the factory. When fees are ready for collection, a set of `hops` (routes) are submitted for each pair detailing how its fee token should be swapped to the WYND token. After necessary `hops` are defined for each fee token, a collection can be done for N tokens in one call.