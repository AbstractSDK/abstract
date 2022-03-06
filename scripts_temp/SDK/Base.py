from re import T
from unicodedata import name
from terra_sdk.core.bank import MsgSend
from terra_sdk.core.auth import StdFee
from terra_sdk.core.wasm import MsgStoreCode, MsgInstantiateContract, MsgExecuteContract
import base64
import json

import pathlib
import sys
from typing import List

from cw_os.contracts.manager import *
from cw_os.contracts.treasury import *
from cw_os.contracts.version_control import *
from cw_os.contracts.os_factory import *
from cw_os.contracts.module_factory import *
from terra_sdk.core.coins import Coin
from pandora_sdk.deploy import get_deployer

mnemonic = "man goddess right advance aim into sentence crime style salad enforce kind matrix inherit omit entry brush never flat strategy entire outside hedgehog umbrella"
# localterra
# deployer = get_deployer(mnemonic=mnemonic, chain_id="columbus-5", fee=None)
# deployer = get_deployer(mnemonic=mnemonic, chain_id="bombay-12", fee=None)
deployer = get_deployer(mnemonic=mnemonic, chain_id="localterra", fee=None)

version_control = VersionControlContract(deployer)
manager = OSManager(deployer)
treasury = TreasuryContract(deployer)
factory = OsFactoryContract(deployer)
module_factory = ModuleFactoryContract(deployer)

create_vc = False
create_manager = False
create_factory = True
create_module_factory = True

if create_vc:
    version_control.upload()
    version_control.instantiate()
    
    version_control.add_module_code_id(name="pandora:terraswap", version= "v0.1.0",code_id= version_control.get("terraswap_dapp", True))
    version_control.add_module_code_id(name="pandora:treasury", version= "v0.1.0",code_id= version_control.get("treasury", True))

if create_manager:
    manager.upload()
    # treasury.upload()
    version_control.add_module_code_id(name="pandora:manager", version= "v0.1.41",code_id= version_control.get("manager", True))

if create_module_factory:
    module_factory.upload()
    module_factory.instantiate()
    module_factory.update_binaries("pandora:terraswap","v0.1.0")

if create_factory:
    # factory.upload()
    factory.instantiate()
    version_control.set_factory()
    
# factory.update_config()

# TODO: add contract_ids to version_control

