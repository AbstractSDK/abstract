# AnsHost asset/contract naming conventions

The ans_host contract stores two types of information.  
1. AssetInfo (cw-asset)
2. Contract Addresses

By providing this naming convention we aim to provide reliable raw-query interfaces to handle low-gas address translation.

## Assets

assets are stored in a map: 
```
use cw-asset::AssetInfo;

pub const ASSET_ADDRESSES: Map<&str, AssetInfo> = Map::new("assets");
```

All key values follow snake_case convention which is relevant for LP tokens. 

Native and cw20 tokens are stored using their ticker as key:

```
"luna": "uluna"
"bluna": "terra1u0t35drzyy0mujj8rkdyzhe264uls4ug3wdp3x"
```

LP tokens are stored by post-fixing the dex identifier,

- tswap (Terraswap)
- astro (Astroport)
- loop (Loop Finance)
- prism (Prism Dex)  

> Abbreviations are used to reduce key size as it effects gas usage.


Here are some examples: 

```
"tswap:luna_ust": "..."  
"astro:luna_ust": "..."  
"junoswap:mbaba_ust":"..."  
```
Notice that the two pool assets are stored in alphabetic order. 


