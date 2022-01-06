from terra_sdk.core.bank import MsgSend
from terra_sdk.core.auth import StdFee
from terra_sdk.core.wasm import MsgStoreCode, MsgInstantiateContract, MsgExecuteContract
import base64
import json

import pathlib
import sys
from typing import List
# temp workaround
sys.path.append('/workspaces/devcontainer/dao-os-SDK/src')
sys.path.append(pathlib.Path(__file__).parent.resolve())

from dao_os.contracts.memory import *
from dao_os.contracts.treasury import *
from dao_os.contracts.vault import *
from terra_sdk.core.coins import Coin
from dao_os.deploy import get_deployer

# mnemonic = "napkin guess language merit split slice source happy field search because volcano staff section depth clay inherit result assist rubber list tilt chef start"
mnemonic = "coin reunion grab unlock jump reason year estate device elevator clean orbit pencil spawn very hope floor actual very clay stereo federal correct beef"

# deployer = get_deployer(mnemonic=mnemonic, chain_id="columbus-5", fee=None)
deployer = get_deployer(mnemonic=mnemonic, chain_id="bombay-12", fee=None)

memory = MemoryContract(deployer)
luna_vault = TreasuryContract(deployer)
liq_contract = VaultContract(deployer)

create = False

if create:
    luna_vault.create()
    liq_contract.create()
    luna_vault.update_vault_assets("luna")
    luna_vault.add_dapp(memory.get("vault_dapp"))

# memory.auto_update_contract_addresses()
# memory.auto_update_asset_addresses()
# memory.query_contracts(["governance"])
# luna_vault.query_total_value()
# liq_contract.provide_liquidity()
liq_contract.withdraw_all()
# liq_contract.query_config()
# liq_contract.set_treasury_addr()
# memory.query_assets(["bluna"]) # , "ust", "whale", "luna_ust"

exit()
