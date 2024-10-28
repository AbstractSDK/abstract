# Abstract IBC Client

The IBC client contract provides a way for Abstract Accounts to create Interchain Abstract Accounts (ICAA) on other chains. It relies on [Polytone](https://github.com/DA0-DA0/polytone) for IBC message forwarding.

Users can enable IBC by calling the `ExecuteMsg::InstallModules` with the `abstract:ibc-client` module ID on their account. This will register the `abstract:ibc_client` with the account so it can be referenced by other applications.

You can learn more about Abstract IBC [in our docs](https://docs.abstract.money/3_framework/8_ibc.html)!
