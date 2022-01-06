import {executeContract} from "./helpers.js";
import { LCDClient, Wallet, LocalTerra} from "@terra-money/terra.js";
import utils from 'web3-utils';

//-----------------------------------------------------

// ------ ExecuteContract :: Function signatures ------
// - updateAirdropConfig
// - claimAirdrop
// - transferUnclaimedWhaleFromAirdropContract
//------------------------------------------------------

//------------------------------------------------------
// ----------- Queries :: Function signatures ----------
// - getAirdropConfig
// - getAirdropState
// - isAirdropClaimed
// - getUserInfo
//------------------------------------------------------


// UPDATE TERRA MERKLE ROOTS : EXECUTE TX
export async function updateAirdropConfig( terra: LocalTerra | LCDClient, wallet:Wallet, airdropContractAdr: string, new_config: any) {
    let resp = await executeContract(terra, wallet, airdropContractAdr, new_config );
}
  

// AIRDROP CLAIM BY TERRA USER : EXECUTE TX
export async function claimAirdrop( terra: LocalTerra | LCDClient, wallet:Wallet, airdropContractAdr: string,  claim_amount: number, merkle_proof: any, root_index: number  ) {
    if ( merkle_proof.length > 1 ) {
      let claim_for_terra_msg = { "claim": {'claim_amount': claim_amount.toString(), 'merkle_proof': merkle_proof, "root_index": root_index }};
        let resp = await executeContract(terra, wallet, airdropContractAdr, claim_for_terra_msg );
        return resp;        
    } else {
        console.log("AIRDROP TERRA CLAIM :: INVALID MERKLE PROOF");
    }
}
  


// TRANSFER WHALE TOKENS : EXECUTE TX
export async function transferUnclaimedWhaleFromAirdropContract( terra: LocalTerra | LCDClient, wallet:Wallet, airdropContractAdr: string, recepient: string, amount: number) {
    try {
        let transfer_whale_msg = { "transfer_unclaimed_tokens": {'recepient': recepient, 'amount': amount.toString() }};
        let resp = await executeContract(terra, wallet, airdropContractAdr, transfer_whale_msg );
        return resp;        
    }
    catch {
        console.log("ERROR IN transferUnclaimedWhaleFromAirdropContract function")
    }        
}


// GET CONFIG : CONTRACT QUERY
export async function getAirdropConfig(  terra: LocalTerra | LCDClient, airdropContractAdr: string) {
    try {
        let res = await terra.wasm.contractQuery(airdropContractAdr, { "config": {} })
        return res;
    }
    catch {
        console.log("ERROR IN getAirdropConfig QUERY")
    }    
}

// GET STATE : CONTRACT QUERY
export async function getAirdropState(  terra: LocalTerra | LCDClient, airdropContractAdr: string) {
    try {
        let res = await terra.wasm.contractQuery(airdropContractAdr, { "state": {} })
        return res;
    }
    catch {
        console.log("ERROR IN getAirdropState QUERY")
    }    
}


// IS CLAIMED : CONTRACT QUERY
export async function isAirdropClaimed(  terra: LocalTerra | LCDClient, airdropContractAdr: string, address: string ) {
    let is_claimed_msg = { "has_user_claimed": {'address': address }};
    try {
        let res = await terra.wasm.contractQuery(airdropContractAdr, is_claimed_msg)
        return res;
    }
    catch {
        console.log("ERROR IN isAirdropClaimed QUERY")
    }
}
  

// USER INFO : CONTRACT QUERY
export async function getUserInfo(  terra: LocalTerra | LCDClient, airdropContractAdr: string, address: string ) {
    let is_claimed_msg = { "user_info": {'address': address }};
    try {
        let res = await terra.wasm.contractQuery(airdropContractAdr, is_claimed_msg)
        return res;
    }
    catch {
        console.log("ERROR IN getUserInfo QUERY")
    }
}
  

// GET NATIVE TOKEN BALANCE
export async function getUserNativeAssetBalance(terra: LocalTerra | LCDClient, native_asset: string, wallet_addr: string) {
    let res = await terra.bank.balance(  wallet_addr );
    let balances = JSON.parse(JSON.parse(JSON.stringify( res )));
    for (let i=0; i<balances.length;i++) {
        if ( balances[i].denom == native_asset ) {
            return balances[i].amount;
        }
    }    
    return 0;
}