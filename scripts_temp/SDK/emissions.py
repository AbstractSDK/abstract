import base64
import json

import pathlib
import sys
import datetime


from terra_sdk.core.auth import StdFee
from pandora_sdk.deploy import get_deployer
from terra_sdk.core.coins import Coin
from pandora_sdk.contracts.emissions import *

#------------------------
#   Run with: $ cd /workspaces/devcontainer/contracts ; /usr/bin/env /bin/python3 -- /workspaces/devcontainer/contracts/scripts/ust_vault.py 
#------------------------

#####################
#   DEPLOY PARAMETERS
#####################

MILLION = 1_000_000

YEAR = 2021
MONTH = 12
DAY = 12
# 5 AM UTC
HOUR = 5
# START_TIME = datetime.datetime(year=YEAR, month=MONTH, day=DAY, hour=HOUR)
START_TIME = datetime.datetime.utcnow() + datetime.timedelta(minutes=10);

# Start time in linux language
BLOCK_START_TIME = (START_TIME - datetime.datetime(1970, 1, 1)).total_seconds()
# 3 day duration (=72h)
DURATION = datetime.timedelta(days=30)
# DURATION = datetime.timedelta(minutes=3)

END_TIME = START_TIME + DURATION

NOW = datetime.datetime.utcnow()
print(f'LBP starts in {START_TIME - NOW} from now.')

print(f'Starts on {START_TIME}')
print(f'Ends on {END_TIME}')

BLOCK_END_TIME = BLOCK_START_TIME + DURATION.total_seconds()

print(BLOCK_END_TIME - BLOCK_START_TIME)
print((END_TIME - START_TIME).total_seconds())
# mnemonic = "napkin guess language merit split slice source happy field search because volcano staff section depth clay inherit result assist rubber list tilt chef start"
mnemonic = "coin reunion grab unlock jump reason year estate device elevator clean orbit pencil spawn very hope floor actual very clay stereo federal correct beef"
# Account that claims the tokens
claimer = "terra10lm49e6ufm8cfpwcmcltvxkv3s6cqeunyjhaj5"
# deployer = get_deployer(mnemonic=mnemonic, chain_id="columbus-5", fee=None)
deployer = get_deployer(mnemonic=mnemonic, chain_id="bombay-12", fee=None)

emissions = Emissions(deployer)
create = True
print(f'gov address: {emissions.get("governance")}')

input("Confirm")

if create:
    # TODO: make multisig owner 
    emissions.create(int(BLOCK_START_TIME), int(DURATION.total_seconds()))

deployer.whale_balance()

emissions.create_vesting(int(50_000*MILLION), int(BLOCK_START_TIME), int(DURATION.total_seconds()), claimer)
   