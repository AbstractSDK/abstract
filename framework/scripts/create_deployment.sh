#!/bin/sh
vc_code_id=
ans_code_id=
account_factory_code_id=
module_factory_code_id=
bs721_profile_code_id=
bs721_marketplace_code_id=
ibc_client_code_id=
ibc_host_code_id=
proxy_code_id=
manager_code_id=

admin_key=""
admin_addr=""
binary=
gas_price="0.05uthiolx"
tx_flags="--from=$admin_key --gas auto --gas-adjustment 2 --gas-prices=$gas_price -y -o json"


# ANS
ans_i=$($binary tx wasm i $ans_code_id '{"admin": "'$admin_addr'"}'  --label="ans_host" --admin $admin_addr  $tx_flags)
ans_hash=$(echo "$ans_i" | jq -r '.txhash');
echo 'waiting for tx to process'
sleep 6;
ans_tx=$(terpd q tx $ans_hash -o json)
ans_addr=$(echo "$ans_tx" | jq -r '.logs[].events[] | select(.type == "instantiate") | .attributes[] | select(.key == "_contract_address") | .value')
echo "ans_addr: $ans_addr"

# Version Control
vc_i=$($binary tx wasm i $vc_code_id '{"admin": "'$admin_addr'", "security_disabled": false}'  --label="abstract_version_control" --admin $admin_addr  $tx_flags)
vc_hash=$(echo "$vc_i" | jq -r '.txhash')
echo 'waiting for tx to process'
sleep 6;
vc_tx=$(terpd q tx $vc_hash -o json)
vc_addr=$(echo "$vc_tx" | jq -r '.logs[].events[] | select(.type == "instantiate") | .attributes[] | select(.key == "_contract_address") | .value')
echo "vc_addr: $vc_addr"

# Module Factory
mf_i=$($binary tx wasm i $module_factory_code_id '{"admin": "'$admin_addr'","version_control_address":"'$vc_addr'","ans_host_address":"'$ans_addr'"}' $tx_flags --label="abstract_module_factory" --admin $admin_addr)
mf_hash=$(echo "$mf_i" | jq -r '.txhash')
echo 'waiting for tx to process'
sleep 6;
mf_tx=$(terpd q tx $mf_hash -o json)
module_factory_addr=$(echo "$mf_tx" | jq -r '.logs[].events[] | select(.type == "instantiate") | .attributes[] | select(.key == "_contract_address") | .value')
echo "module_factory_addr: $module_factory_addr"

# Account Factory
af_i=$($binary tx wasm i $account_factory_code_id '{"admin": "'$admin_addr'", "version_control_address":"'$vc_addr'","ans_host_address":"'$ans_addr'", "module_factory_address":"'$module_factory_addr'","min_name_length": 3, "max_name_length":128, "base_price": "10"}'  --label="abstract_account_factory" --admin $admin_addr  $tx_flags)
af_hash=$(echo "$af_i" | jq -r '.txhash')
echo 'waiting for tx to process'
sleep 6;
af_tx=$(terpd q tx $af_hash -o json)
account_factory_addr=$(echo "$af_tx" | jq -r '.logs[].events[] | select(.type == "instantiate") | .attributes[] | select(.key == "_contract_address") | .value')
echo "account_factory_addr: $account_factory_addr"

# Bs Profile
bs_profile_i=$($binary tx wasm i $bs721_profile_code_id '{"base_init_msg": {"name":"test","symbol":"TEST", "minter":"'$account_factory_addr'", "collection_info":{"creator":"'$admin_addr'","description":"test description","image":"https://www.testimageurl.com", "external_link":"https://www.beautiful.network"}}}' $tx_flags --label="bs721_profile" --admin $admin_addr )
bs_profile_hash=$(echo "$bs_profile_i" | jq -r '.txhash')
echo 'waiting for tx to process'
sleep 6;
bs_tx=$(terpd q tx $bs_profile_hash -o json)
bs721_profile_addr=$(echo "$bs_tx" | jq -r '.logs[].events[] | select(.type == "instantiate") | .attributes[] | select(.key == "_contract_address") | .value')
echo "bs721_profile_addr: $bs721_profile_addr"

# Marketplace 
marketplace_i=$($binary tx wasm i $bs721_marketplace_code_id '{"trading_fee_bps":25,"min_price": "100", "ask_interval": 100, "factory":"'$account_factory_addr'","collection":"'$bs721_profile_addr'"}' --label="profile marketplace" --admin $admin_addr  $tx_flags)
marketplace_hash=$(echo "$marketplace_i" | jq -r '.txhash')
echo 'waiting for tx to process'
sleep 6;
m_tx=$(terpd q tx $marketplace_hash -o json)
bs721_marketplace_addr=$(echo "$m_tx" | jq -r '.logs[].events[] | select(.type == "instantiate") | .attributes[] | select(.key == "_contract_address") | .value')
echo "bs721_marketplace_addr: $bs721_marketplace_addr"

# IBC Client
ibc_client_i=$($binary tx wasm i $ibc_client_code_id '{"ans_host_address":"'$ans_addr'","version_control_address":"'$vc_addr'"}' --admin $admin_addr  $tx_flags --label="ibc_client" $tx_flags)
ibc_client_hash=$(echo "$ibc_client_i" | jq -r '.txhash')
echo 'waiting for tx to process'
sleep 6;
ibc_c_tx=$(terpd q tx $ibc_client_hash -o json)
ibc_client_addr=$(echo "$m_tx" | jq -r '.logs[].events[] | select(.type == "instantiate") | .attributes[] | select(.key == "_contract_address") | .value')
echo "ibc_client_addr: $ibc_client_addr"

# IBC Host
ibc_host_i=$($binary tx wasm i $ibc_host_code_id '{"ans_host_address":"'$ans_addr'","account_factory_address":"'$account_factory_addr'","version_control_address":"'$vc_addr'"}' --admin $admin_addr  $tx_flags --label="ibc_host" $tx_flags)
ibc_host_hash=$(echo "$ibc_host_i" | jq -r '.txhash')
echo 'waiting for tx to process'
sleep 6;
ibc_h_tx=$(terpd q tx $ibc_host_hash -o json)
ibc_host_addr=$(echo "$m_tx" | jq -r '.logs[].events[] | select(.type == "instantiate") | .attributes[] | select(.key == "_contract_address") | .value')
echo "ibc_host_addr: $ibc_host_addr"

# Update VC config
$binary tx wasm e $vc_addr '{"update_config":{"account_factory_address":"'$account_factory_addr'"}}' $tx_flags
echo 'waiting for tx to process'
sleep 6;

# Propose Modules to VC
MSG=$(cat <<EOF
{
    "propose_modules": {"modules": [
        [{"name": "manager","namespace":"abstract","version": {"version": "0.22.1"}},{"account_base": $manager_code_id}],
        [{"name": "proxy","namespace": "abstract","version": {"version": "0.22.1"}},{"account_base": $proxy_code_id}],
        [{"name": "ans-host","namespace": "abstract","version": {"version": "0.22.1"}},{"native": "$ans_addr"}],
        [{"name": "version-control","namespace": "abstract","version": {"version": "0.22.1"}},{"native": "$vc_addr"}],
        [{"name": "account-factory","namespace": "abstract","version": {"version": "0.22.1"}},{"native": "$account_factory_addr"}],
        [{"name": "module-factory","namespace": "abstract", "version": {"version": "0.22.1"}},{"native": "$module_factory_addr"}],
        [{"name": "ibc-client","namespace": "abstract","version": {"version": "0.22.1"}},{"native": "$ibc_client_addr"}],
        [{"name": "ibc-host","namespace": "abstract","version": {"version": "0.22.1"}},{"native": "$ibc_host_addr"}],
        [{"name": "bs721-profile","namespace": "abstract","version": {"version": "0.22.1"}},{"native": "$bs721_profile_addr"}],
        [{"name": "profile-marketplace","namespace": "abstract","version": {"version": "0.22.1"}},{"native": "$bs721_marketplace_addr"}]
    ]}
}
EOF
)
echo $MSG
$binary tx wasm e $vc_addr "$MSG" $tx_flags
echo 'waiting for tx to process'
sleep 6;
# Approve Modules to VC
$binary tx wasm e $vc_addr '{"approve_or_reject_modules": {"approves": [{"name": "manager","namespace": "abstract","version": {"version": "0.22.1"}},{"name": "proxy","namespace": "abstract","version": {"version": "0.22.1"}},{"name": "ans-host","namespace": "abstract","version": {"version": "0.22.1"}},{"name": "version-control","namespace": "abstract","version": {"version": "0.22.1"}},{"name": "account-factory","namespace": "abstract","version": {"version": "0.22.1"}},{"name": "module-factory","namespace": "abstract","version": {"version": "0.22.1"}},{"name": "ibc-client","namespace": "abstract","version": {"version": "0.22.1"}},{"name": "ibc-host","namespace": "abstract","version": {"version": "0.22.1"}},{"name": "bs721-profile","namespace": "abstract","version": {"version": "0.22.1"}},{"name": "profile-marketplace","namespace": "abstract","version": {"version": "0.22.1"}}],"rejects": []}}' $tx_flags
echo 'waiting for tx to process'
sleep 6;

# Update Account Factory Config 
$binary tx wasm e $account_factory_addr '{"update_config":{"ibc_host":"'$ibc_host_addr'"}}' $tx_flags
echo 'waiting for tx to process'
sleep 6;

# Setup Profile Infra on Account Factory
$binary tx wasm e $account_factory_addr '{"setup_profile_infra":{"marketplace_addr":"'$bs721_marketplace_addr'","profile_addr":"'$bs721_profile_addr'"}}' $tx_flags 
echo 'waiting for tx to process'
sleep 6;

## Create Account 
 $binary tx wasm e $account_factory_addr '{"create_account": {"governance":{"Monarchy":{"monarch":"'$admin_addr'"}},"name":"first-os","install_modules":[],"bs_profile":"the-monk-on-iron-mountain"}}'$tx_flags

## Query Profile Collection 
$binary q wasm contract-state smart $bs721_profile_addr '{"all_tokens":{}}'

## Query Marketplace 
$binary q wasm contract-state smart $bs721_marketplace_addr '{"ask":{"token_id":"the-monk-on-iron-mountain"}}'