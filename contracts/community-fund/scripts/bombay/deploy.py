import pathlib
import sys
# temp workaround
sys.path.append('/workspaces/devcontainer/terra-sdk-python')
sys.path.append('/workspaces/devcontainer/White-Whale-SDK/src')
sys.path.append(pathlib.Path(__file__).parent.resolve())

from terra_sdk.core.auth import StdFee
from white_whale.deploy import get_deployer
from white_whale.address.bombay.anchor import anchor_money_market, aust
from white_whale.address.bombay.white_whale import whale_token, whale_ust_pool, governance, community_fund

import pathlib
import sys
sys.path.append(pathlib.Path(__file__).parent.resolve())

mnemonic = "coin reunion grab unlock jump reason year estate device elevator clean orbit pencil spawn very hope floor actual very clay stereo federal correct beef"
std_fee = StdFee(5000000, "1500000uusd")

deployer = get_deployer(mnemonic=mnemonic, chain_id="bombay-12", fee=std_fee)

print("store contract")
code_id = deployer.store_contract(contract_name="community_fund")
print(f"stored {code_id}")
print("instantiate contract")
contract_address = deployer.instantiate_contract(code_id=code_id, init_msg={
    "whale_token_addr": whale_token,
    "whale_pair_addr": whale_ust_pool,
    "anchor_money_market_addr": anchor_money_market,
    "aust_addr": aust,
    "anchor_deposit_threshold": str(int(10)*int(10**6)),
    "anchor_withdraw_threshold": str(int(1)*int(10**4)),
})
print(f'instantiated {contract_address}')

# result = deployer.execute_contract(community_fund, {
#     "update_admin": {
#         "admin": governance
#     }
# })
# print(result)