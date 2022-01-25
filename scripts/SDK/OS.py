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
from terra_sdk.core.coins import Coin
from cw_os.deploy import get_deployer

mnemonic = "man goddess right advance aim into sentence crime style salad enforce kind matrix inherit omit entry brush never flat strategy entire outside hedgehog umbrella"

# deployer = get_deployer(mnemonic=mnemonic, chain_id="columbus-5", fee=None)
# deployer = get_deployer(mnemonic=mnemonic, chain_id="bombay-12", fee=None)
deployer = get_deployer(mnemonic=mnemonic, chain_id="localterra", fee=None)

version_control = VersionControlContract(deployer)
manager = OSManager(deployer)
treasury = TreasuryContract(deployer)
factory = OsFactoryContract(deployer)

create_os = True

if create_os:
    # version_control.set_admin(deployer.wallet.key.acc_address)
    factory.create_os()

factory.query_config()

# TODO: add contract_ids to version_control

