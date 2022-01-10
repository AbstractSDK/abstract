from terra_sdk.core.bank import MsgSend
from terra_sdk.core.auth import StdFee
from terra_sdk.core.wasm import MsgStoreCode, MsgInstantiateContract, MsgExecuteContract
import base64
import json

import pathlib
import sys
from typing import List

from cw_os.contracts.version_control import *
from terra_sdk.core.coins import Coin
from cw_os.deploy import get_deployer

mnemonic = "man goddess right advance aim into sentence crime style salad enforce kind matrix inherit omit entry brush never flat strategy entire outside hedgehog umbrella"

# deployer = get_deployer(mnemonic=mnemonic, chain_id="columbus-5", fee=None)
deployer = get_deployer(mnemonic=mnemonic, chain_id="bombay-12", fee=None)

version_control = VersionControlContract(deployer)

create = True

if create:
    version_control.create()
