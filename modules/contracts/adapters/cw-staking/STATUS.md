# Adapter Status

This document describes the status of the staking adapter's integrations with different external systems.

| Protocol | Implementation | Execution Tests | Query Tests | Notes |
| --- | --- | --- | --- | --- |
| Osmosis | ✅ | ✅ | ❌ | Query failing because of missing whitelist. |
| Astroport | ✅ | ✅ | ✅ | |
| Wynd | ✅ | ✅ | ❌ | |
| Bow | ✅ | ✅ | ✅ | Creating pool not possible because it uses custom module. |
| Astrovault | ✅ | ✅ | ✅ | Astrovault uses custom archway module to create pool, so we rely on existing pools. |
