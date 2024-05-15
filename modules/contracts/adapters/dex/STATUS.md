# Adapter Status

This document describes the status of the dex adapter's integrations with different external systems.

| Protocol | Implementation | Execution Tests | Query Tests | Notes |
| --- | --- | --- | --- | --- |
| Osmosis | ✅ | ✅ | ❌ | |
| Astroport | ✅ | ✅ | ✅ | |
| Wynd | ✅ | ✅ | ❌ | |
| Bow | ✅ | ❌ | ✅ | Liquidity tests not implemented because it uses custom module. |
| Astrovault | ✅ | ✅ | ✅ | Integration: Archway wasm size cannot be longer than 819200 bytes, which is lower than needed. To compensate it we sacrificed not commonly used `provide_liquidity_symmetric` method. Testing: Astrovault uses custom archway module to create pool, so we rely on existing pools for testing. |
