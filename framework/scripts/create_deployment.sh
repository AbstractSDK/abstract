#!/bin/sh
account_factory_code_id=1
ans_code_id=2
ibc_client_code_id=3
ibc_host_code_id=4
manager_code_id=5
module_factory_code_id=6
proxy_code_id=7
vc_code_id=8
bs721_profile_code_id=9
bs721_marketplace_code_id=10
bs_profile_minter_code_id=11

admin_key=""
bidder_key=""
admin_addr=""
bidder_addr=""
token_id="the-monk-on-iron-mountain"
binary=
chain_id=
gas_price=""
tx_flags="--from=$admin_key  --chain-id $chain_id --gas auto --gas-adjustment 2 --gas-prices=$gas_price -y -o json"
tx_flags_2="--from=$bidder_key --chain-id $chain_id --gas auto --gas-adjustment 2 --gas-prices=$gas_price -y -o json"

version='{"version": "0.22.1"}'

# fund second account 
echo 'fund second account'
fund=$($binary tx bank send $admin_addr $bidder_addr 1000000000ubtsg --gas auto -y -o json )
fund_hash=$(echo "$fund" | jq -r '.txhash');
echo 'waiting for tx to process'
sleep 6;
fund_tx=$($binary q tx $fund_hash -o json)

# ANS
echo 'Creating ANS'
ans_i=$($binary tx wasm i $ans_code_id '{"admin": "'$admin_addr'"}'  --label="ans_host" --admin $admin_addr  $tx_flags)
ans_hash=$(echo "$ans_i" | jq -r '.txhash');
echo 'waiting for tx to process'
sleep 6;
ans_tx=$($binary q tx $ans_hash -o json)
ans_addr=$(echo "$ans_tx" | jq -r '.logs[].events[] | select(.type == "instantiate") | .attributes[] | select(.key == "_contract_address") | .value')
echo "ans_addr: $ans_addr"

# Version Control
echo 'Creating Version Control'
vc_i=$($binary tx wasm i $vc_code_id '{"admin": "'$admin_addr'", "security_disabled": false}'  --label="abstract_version_control" --admin $admin_addr  $tx_flags)
vc_hash=$(echo "$vc_i" | jq -r '.txhash')
echo 'waiting for tx to process'
sleep 6;
vc_tx=$($binary q tx $vc_hash -o json)
vc_addr=$(echo "$vc_tx" | jq -r '.logs[].events[] | select(.type == "instantiate") | .attributes[] | select(.key == "_contract_address") | .value')
echo "vc_addr: $vc_addr"

# Module Factory
echo 'Creating Module Factory'
mf_i=$($binary tx wasm i $module_factory_code_id '{"admin": "'$admin_addr'","version_control_address":"'$vc_addr'","ans_host_address":"'$ans_addr'"}' $tx_flags --label="abstract_module_factory" --admin $admin_addr)
mf_hash=$(echo "$mf_i" | jq -r '.txhash')
echo 'waiting for tx to process'
sleep 6;
mf_tx=$($binary q tx $mf_hash -o json)
module_factory_addr=$(echo "$mf_tx" | jq -r '.logs[].events[] | select(.type == "instantiate") | .attributes[] | select(.key == "_contract_address") | .value')
echo "module_factory_addr: $module_factory_addr"

# Account Factory
echo 'Creating Account Factory'
af_i=$($binary tx wasm i $account_factory_code_id '{"admin": "'$admin_addr'", "version_control_address":"'$vc_addr'","ans_host_address":"'$ans_addr'", "module_factory_address":"'$module_factory_addr'"}'  --label="abstract_account_factory" --admin $admin_addr  $tx_flags)
af_hash=$(echo "$af_i" | jq -r '.txhash')
echo 'waiting for tx to process'
sleep 6;
af_tx=$($binary q tx $af_hash -o json)
account_factory_addr=$(echo "$af_tx" | jq -r '.logs[].events[] | select(.type == "instantiate") | .attributes[] | select(.key == "_contract_address") | .value')
echo "account_factory_addr: $account_factory_addr"

# IBC Client
echo 'Creating IBC Client'
ibc_client_i=$($binary tx wasm i $ibc_client_code_id '{"ans_host_address":"'$ans_addr'","version_control_address":"'$vc_addr'"}' --admin $admin_addr  $tx_flags --label="ibc_client")
ibc_client_hash=$(echo "$ibc_client_i" | jq -r '.txhash')
echo 'waiting for tx to process'
sleep 6;
ibc_c_tx=$($binary q tx $ibc_client_hash -o json)
ibc_client_addr=$(echo "$ibc_c_tx" | jq -r '.logs[].events[] | select(.type == "instantiate") | .attributes[] | select(.key == "_contract_address") | .value')
echo "ibc_client_addr: $ibc_client_addr"

# IBC Host
echo 'Creating IBC Host'
ibc_host_i=$($binary tx wasm i $ibc_host_code_id '{"ans_host_address":"'$ans_addr'","account_factory_address":"'$account_factory_addr'","version_control_address":"'$vc_addr'"}' --admin $admin_addr  $tx_flags --label="ibc_host")
ibc_host_hash=$(echo "$ibc_host_i" | jq -r '.txhash')
echo 'waiting for tx to process'
sleep 6;
ibc_h_tx=$($binary q tx $ibc_host_hash -o json)
ibc_host_addr=$(echo "$ibc_h_tx" | jq -r '.logs[].events[] | select(.type == "instantiate") | .attributes[] | select(.key == "_contract_address") | .value')
echo "ibc_host_addr: $ibc_host_addr"

# Update VC config
echo 'Updating VC Config'
$binary tx wasm e $vc_addr '{"update_config":{"account_factory_address":"'$account_factory_addr'"}}' $tx_flags
echo 'waiting for tx to process'
sleep 6;

# Propose Modules to VC
echo 'Proposing Modules to VC'
MSG=$(cat <<EOF
{
    "propose_modules": {"modules": [
        [{"name": "manager","namespace":"abstract","version": $version},{"account_base": $manager_code_id}],
        [{"name": "proxy","namespace": "abstract","version": $version},{"account_base": $proxy_code_id}],
        [{"name": "ans-host","namespace": "abstract","version": $version},{"native": "$ans_addr"}],
        [{"name": "version-control","namespace": "abstract","version": $version},{"native": "$vc_addr"}],
        [{"name": "account-factory","namespace": "abstract","version": $version},{"native": "$account_factory_addr"}],
        [{"name": "module-factory","namespace": "abstract", "version": $version},{"native": "$module_factory_addr"}],
        [{"name": "ibc-client","namespace": "abstract","version": $version},{"native": "$ibc_client_addr"}],
        [{"name": "ibc-host","namespace": "abstract","version": $version},{"native": "$ibc_host_addr"}]
    ]}
}
EOF
)
echo $MSG
$binary tx wasm e $vc_addr "$MSG" $tx_flags
echo 'waiting for tx to process'
sleep 6;
# Approve Modules to VC
echo 'Approve Modules to VC'
MSG2=$(cat <<EOF
{"approve_or_reject_modules": {
    "approves": [
        {"name": "manager","namespace": "abstract","version": $version},
        {"name": "proxy","namespace": "abstract","version": $version},
        {"name": "ans-host","namespace": "abstract","version": $version},
        {"name": "version-control","namespace": "abstract","version": $version},
        {"name": "account-factory","namespace": "abstract","version": $version},
        {"name": "module-factory","namespace": "abstract","version": $version},
        {"name": "ibc-client","namespace": "abstract","version": $version},
        {"name": "ibc-host","namespace": "abstract","version": $version}
    ],
    "rejects": []
    }
}
EOF
)
$binary tx wasm e $vc_addr "$MSG2" $tx_flags
echo 'waiting for tx to process'
sleep 6;

# Update Account Factory Config 
echo 'Update Account Factory'
$binary tx wasm e $account_factory_addr '{"update_config":{"ibc_host":"'$ibc_host_addr'"}}' $tx_flags
echo 'waiting for tx to process'
sleep 6;

## Create Account 
echo 'Create Account 1'
admin_tx=$($binary tx wasm e $account_factory_addr '{"create_account": {"governance":{"NFT":{"collection_addr":"'$bs721_profile_addr'", "token_id":"'$token_id'"}},"name":"first-os","install_modules":[]}}' $tx_flags --amount 10000000ubtsg )
echo 'waiting for tx to process'
sleep 6;
admin_account_tx=$(echo "$admin_tx" | jq -r '.txhash')
echo $admin_account_tx
admin_query=$($binary q tx $admin_account_tx -o json)
admin_manager_addr=$(echo "$admin_query" | jq -r '.logs[].events[] | select(.type == "wasm-abstract") | .attributes[] | select(.key == "manager_address") | .value')
admin_proxy_addr=$(echo "$admin_query" | jq -r '.logs[].events[] | select(.type == "wasm-abstract") | .attributes[] | select(.key == "proxy_address") | .value')
echo 'admin_manager_addr: '$admin_manager_addr''
echo 'admin_proxy_addr: '$admin_proxy_addr''

# Call Functions Through Smart Contract Account
echo 'use account contracts to call msg burning tokens'
$binary q bank balances $admin_proxy_addr
burn_msg_binary='{"module_action":{"msgs":[{"bank":{"burn":{"amount":[{"amount":"100","denom":"ubtsg"}]}}}]}}' 
burn_binary=$(echo $burn_msg_binary | jq -c . | base64)

burn_msg_tx=$($binary tx wasm e $admin_manager_addr '{"exec_on_module":{"module_id":"abstract:proxy","exec_msg":"'$burn_binary'"}}' $tx_flags)
burn_tx=$(echo "$burn_msg_tx" | jq -r '.txhash')

echo $burn_tx
# Query Proxy balance
sleep 6;
$binary q bank balances $admin_proxy_addr