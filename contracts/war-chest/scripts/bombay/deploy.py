import pathlib
import sys
# temp workaround
sys.path.append('/workspaces/devcontainer/terra-sdk-python')
sys.path.append('/workspaces/devcontainer/White-Whale-SDK/src')
sys.path.append(pathlib.Path(__file__).parent.resolve())

from terra_sdk.core.auth import StdFee
from white_whale.deploy import get_deployer
from white_whale.address.bombay.white_whale import whale_token, governance

import pathlib
import sys
sys.path.append(pathlib.Path(__file__).parent.resolve())

mnemonic = "main jar girl opinion train type cycle blood marble kitchen april champion amount engine crumble tunnel model vicious system student hood fee curious traffic"
std_fee = StdFee(5000000, "1500000uusd")

deployer = get_deployer(mnemonic=mnemonic, chain_id="bombay-12", fee=std_fee)

print("store contract")
code_id = deployer.store_contract(contract_name="war_chest")
print(f"stored {code_id}")
print("instantiate contract")
contract_address = deployer.instantiate_contract(code_id=code_id, init_msg={
    "admin_addr": governance,
    "whale_token_addr": whale_token,
    "spend_limit": "1000000000",
})
print(f'instantiated {contract_address}')