import pathlib
import sys
# temp workaround
sys.path.append('/workspaces/devcontainer/terra-sdk-python')
sys.path.append('/workspaces/devcontainer/White-Whale-SDK/src')
sys.path.append(pathlib.Path(__file__).parent.resolve())

from terra_sdk.core.auth import StdFee
from white_whale.deploy import get_deployer
from white_whale.address.bombay.white_whale import whale_ust_pool, whale_token

import pathlib
import sys
sys.path.append(pathlib.Path(__file__).parent.resolve())

mnemonic = "<MNEMONIC>"
std_fee = StdFee(5000000, "1500000uusd")

deployer = get_deployer(mnemonic=mnemonic, chain_id="bombay-12", fee=std_fee)

ust_info = { "native_token": { "denom": "uusd" } }
whale_info = { "token": { "contract_addr": whale_token }}

print("store contract")
code_id = deployer.store_contract(contract_name="terraswap_wrapper")
print(f"stored {code_id}")
print("instantiate contract")
max_deposit = int(10**9)
min_profit = int(10**4)
contract_address = deployer.instantiate_contract(code_id=code_id, init_msg={
    "terraswap_pool_addr": whale_ust_pool,
    "trader": deployer.wallet.key.acc_address,
    "max_deposit": {
        "info": ust_info,
        "amount": str(max_deposit)
    },
    "min_profit": {
        "info": ust_info,
        "amount": str(min_profit)
    },
    "slippage": "0.01"
})
print(f'instantiated {contract_address}')