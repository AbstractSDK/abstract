import base64
import json

import pathlib
import sys
# temp workaround
sys.path.append('/workspaces/devcontainer/dao-os-SDK/src')
sys.path.append(pathlib.Path(__file__).parent.resolve())

from terra_sdk.core.auth import StdFee
from dao_os.deploy import get_deployer
from terra_sdk.core.coins import Coin
from dao_os.contracts.governance import *

#------------------------
#   Run with: $ cd /workspaces/devcontainer/contracts ; /usr/bin/env /bin/python3 -- /workspaces/devcontainer/contracts/scripts/ust_vault.py 
#------------------------

# mnemonic = "napkin guess language merit split slice source happy field search because volcano staff section depth clay inherit result assist rubber list tilt chef start"
mnemonic = "coin reunion grab unlock jump reason year estate device elevator clean orbit pencil spawn very hope floor actual very clay stereo federal correct beef"

# deployer = get_deployer(mnemonic=mnemonic, chain_id="columbus-5", fee=None)
deployer = get_deployer(mnemonic=mnemonic, chain_id="bombay-12", fee=None)

gov = Governance(deployer)
create = False

if create:
    gov.create()
    gov.set_token()

deployer.whale_balance()
# gov.stake(1000)
gov.get_staked_amount()
gov.create_poll("")
# gov.unstake_all()

# gov.create_poll()
   