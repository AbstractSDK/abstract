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
from terra_sdk.core.coins import Coin
from cw_os.deploy import get_deployer

mnemonic = "man goddess right advance aim into sentence crime style salad enforce kind matrix inherit omit entry brush never flat strategy entire outside hedgehog umbrella"

# deployer = get_deployer(mnemonic=mnemonic, chain_id="columbus-5", fee=None)
deployer = get_deployer(mnemonic=mnemonic, chain_id="bombay-12", fee=None)

version_control = VersionControlContract(deployer)
manager = OSManager(deployer)
treasury = TreasuryContract(deployer)

create_base = False
create_os = False

if create_base:
    version_control.upload()
    version_control.instantiate()

if create_os:
    manager.upload()
    treasury.upload()
    manager.instantiate()
    version_control.add_os(manager.query_os_id())
    treasury.instantiate()
    manager.add_module("treasury",manager.get("treasury"))


# TODO: add contract_ids to version_control

