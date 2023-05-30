# Upgrade Modules
One of the key strengths of Abstract is that it is designed to minimize your maintenance workload while maximizing the control you have over your infrastructure.

Abstract manages module upgrades for you, ensuring your infrastructure remains intact and your workflows continue to function smoothly through every upgrade. This process is carried out in a manner that consistently maintains the integrity and security of your system.
```mermaid
sequenceDiagram
  autonumber
  actor U as User
  participant M as Manager
  participant VC as Version Control
  participant P as Proxy

  U->>M: Upgrade
  loop for each module
    M-->>VC: Query reference
    alt adapter
        VC-->>M: Return address
        M->>M: Update module address
        M->>+P: Remove old adapter from allowlist
        M->>P: Add new adapter to allowlist
        deactivate P
    else app / standalone
        VC-->>M: Return code_id
        M->>M: Migrate module to new code_id
    end
  end

  alt
    M->M: Migrate self
   end
  M->M: Update dependencies
  M-->M: Check dependencies  
```
