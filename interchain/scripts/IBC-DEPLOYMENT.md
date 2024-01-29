# Deployment status of Abstract IBC

| From\To  | Osmosis | Archway | Terra | Juno | Neutron |
|----------|---------|---------|-------|------|---------|
| Osmosis  |   ❌    |         |       |      |         |
| Archway  |   ✅*   |   ❌    |  ✅   |  ✅  |   ✅    |
| Terra    |   ✅*   |   ✅    |  ❌   |  ✅* |   ✅    |
| Juno     |   ✅    |   ✅    |  ✅   |  ❌  |   ❌*   |
| Neutron  |   ✅    |   ✅*   |  ✅   |  ❌* |   ❌    |

❌: - No reason to deploy
❌*:  - For Neutron - Juno, there is no polytone connection (because there is no active IBC connection it seems)
✅ : connection was successfuly created.
✅* manual relaying was needed for abstract IBC channel creation.


# Test status of Abstract IBC

| From\To  | Osmosis | Archway | Terra | Juno | Neutron |
|----------|---------|---------|-------|------|---------|
| Osmosis  |   ❌    |   ❌*   |  ❌*  |  ❌* |   ❌*   |
| Archway  |   ✅*   |   ❌    |  ✅   |  ✅  |   ✅**  |
| Terra    |   ✅    |   ✅*   |  ❌   |  ✅* |   ✅    |
| Juno     |   ✅    |   ✅    |  ✅   |  ❌  |   ❌*   |
| Neutron  |   ✅*   |   ✅*   |  ✅   |  ❌* |   ❌    |

❌: - No reason to deploy
❌*:  No active connection
✅:  remote account successfully created
✅*: Needed manual relaying
✅**: Needed manual relaying maybe because of too high gas
