from terra_sdk.client.lcd import LCDClient, Wallet
from terra_sdk.key.mnemonic import MnemonicKey
from terra_sdk.core.coins import Coins

from terra_sdk.util.contract import read_file_as_b64, get_code_id, get_contract_address
from terra_sdk.core.auth import StdFee
from terra_sdk.core.wasm import MsgStoreCode, MsgInstantiateContract, MsgExecuteContract
from terra_sdk.core.bank import MsgSend

import pathlib
import sys
sys.path.append(pathlib.Path(__file__).parent.resolve())

client = LCDClient(url="https://tequila-lcd.terra.dev", chain_id="tequila-0004")
mnemonic = "napkin guess language merit split slice source happy field search because volcano staff section depth clay inherit result assist rubber list tilt chef start"
deployer = Wallet(lcd=client, key=MnemonicKey(mnemonic))
std_fee = StdFee(4000000, "1000000uusd")


def send_msg(msg):
    tx = deployer.create_and_sign_tx(
        msgs=[msg], fee=std_fee
    )
    return client.tx.broadcast(tx)

def execute_contract(contract_addr: str, execute_msg):
    msg = MsgExecuteContract(
        sender=deployer.key.acc_address,
        contract=contract_addr,
        execute_msg=execute_msg
    )
    return send_msg(msg)

def send_funds(receiver: str, amount: int):
    msg = MsgSend(
        from_address=deployer.key.acc_address,
        to_address=receiver,
        amount=Coins(str(amount))
    )
    return send_msg(msg)

GGY_ADDRESS = "terra1gdj4adgs90avvrddf4v4ft2zj526y3uwn4flrt"
burn_address = "terra1438rqfx8r8y3kxrqhpr7le4ewppdssn0x593k0"

# transfer_amount = 1
# result = execute_contract(contract_addr=GGY_ADDRESS, execute_msg={
#     "transfer": {
#         "recipient": burn_address,
#         "amount": str(transfer_amount)
#     }
# })
# print(result)
result = execute_contract(contract_addr=burn_address, execute_msg={
    "burn" : {  }
})
print(result)
result = client.wasm.contract_query(GGY_ADDRESS, {
    "balance": {"address": burn_address}
})
print(result)
result = client.wasm.contract_query(GGY_ADDRESS, {
    "token_info": {}
})
print(result)

