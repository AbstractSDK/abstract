import pathlib
import sys
# temp workaround
sys.path.append('/workspaces/devcontainer/terra-sdk-python')
sys.path.append('/workspaces/devcontainer/White-Whale-SDK/src')
sys.path.append(pathlib.Path(__file__).parent.resolve())

from terra_sdk.core.auth import StdFee
from white_whale.deploy import get_deployer
from white_whale.address.bombay.white_whale import whale_token

import pathlib
import sys
sys.path.append(pathlib.Path(__file__).parent.resolve())

mnemonic = "main jar girl opinion train type cycle blood marble kitchen april champion amount engine crumble tunnel model vicious system student hood fee curious traffic"
std_fee = StdFee(5000000, "1500000uusd")

deployer = get_deployer(mnemonic=mnemonic, chain_id="bombay-12", fee=std_fee)

###
print("store contract")
code_id = deployer.store_contract(contract_name="gov")
print(f"stored {code_id}")
print("instantiate contract")
contract_address = deployer.instantiate_contract(code_id=code_id, init_msg={
    "quorum": "0.3",
    "threshold": "0.5",
    "timelock_period": 10000,
    "voting_period": 10000,
    "expiration_period": 20000,
    "proposal_deposit": "1000",
    "snapshot_period": 10
})
print(f'instantiated {contract_address}')

result = deployer.execute_contract(contract_address, execute_msg={
    "register_contracts": { "whale_token": whale_token }
})
print(result)