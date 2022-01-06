from pathlib import Path
import shutil, os

# This script is to be ran from the dapps folder, the root where the template and dapps live.
# In case you need to modify anything in a dapp testing mocks, consider modifying the base_mocks
# on the dapp-template contract and then use this script to clone it accross dapps.
# $ python3 ../../../scripts/dapp_base/clone_dapp_template_test_mocks.py
src = 'dapp-template/src/tests/base_mocks'
dapp_template='dapp-template'
dapps=next(os.walk('.'))[1]

files=os.listdir(src)

for dapp in dapps:
    if (dapp != dapp_template):
        for fname in files:
            shutil.copy2(os.path.join(src,fname), dapp + '/src/tests/base_mocks')
