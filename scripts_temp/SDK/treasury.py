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

from cw_os.contracts.terraswap_dapp import *
from cw_os.contracts.proxy import *
from terra_sdk.core.coins import Coin
from cw_os.deploy import get_deployer

def execute_on_proxy_msg(msgs: any, coins: List[Coin]):
    msg = MsgExecuteContract(
        deployer.wallet.key.acc_address,
        proxy.address,
        {
            "trader_action": {
                "msgs": msgs
            }
        },
        coins,
    )
    return msg


# mnemonic = "napkin guess language merit split slice source happy field search because volcano staff section depth clay inherit result assist rubber list tilt chef start"
mnemonic = "coin reunion grab unlock jump reason year estate device elevator clean orbit pencil spawn very hope floor actual very clay stereo federal correct beef"

# deployer = get_deployer(mnemonic=mnemonic, chain_id="columbus-5", fee=None)
# deployer = get_deployer(mnemonic=mnemonic, chain_id="bombay-12", fee=None)
deployer = get_deployer(mnemonic=mnemonic, chain_id="localterra", fee=None)

proxy = TreasuryContract(deployer)
terraswap_dapp = TerraswapDAppContract(deployer)

create = True

if create:
    # proxy.create()
    terraswap_dapp.upload()
    terraswap_dapp.instantiate()
    # proxy.add_dapp(terraswap_dapp.address)
    # proxy.add_dapp(deployer.wallet.key.acc_address)

# exit()
# print(deployer.wallet.key.acc_address)
# proxy.update_vault_assets()
# terraswap_dapp.query_config()
# terraswap_dapp.auto_update_address_book()

# terraswap_dapp.detailed_provide_liquidity("lbp_pair", [("whale", str(int(1000000000))), ("ust", str(int(100000000)))], None)
# exit()
# proxy.query_holding_amount("uluna")
# proxy.send_asset("uluna", 10000, "terra1khmttxmtsmt0983ggwcufalxkn07l4yj5thu3h")
# proxy.query_vault_asset("uluna")
# terraswap_dapp.swap("ust", "lbp_pair", int(100000))
# terraswap_dapp.provide_liquidity("lbp_pair", "whale", int(9000000))
# proxy.query_holding_value("uluna")

# LBP token id
# terraswap_dapp.withdraw_liquidity("lbp", 315511529)

exit()
