import chalk from "chalk";
import { join } from "path"
import { LocalTerra, Wallet } from "@terra-money/terra.js";
import { expect } from "chai";
import { deployContract, transferCW20Tokens, getCW20Balance } from "./helpers/helpers.js";
  import {updateAirdropConfig, claimAirdrop, transferUnclaimedWhaleFromAirdropContract
    ,getAirdropConfig, getAirdropState, isAirdropClaimed, getUserInfo,  }  from "./helpers/airdrop_helpers.js";
import  {Terra_Merkle_Tree}  from "./helpers/merkle_tree.js";

//----------------------------------------------------------------------------------------
// Variables
//----------------------------------------------------------------------------------------

const ARTIFACTS_PATH = "../artifacts"
const terra = new LocalTerra();

const deployer = terra.wallets.test1;

const terra_user_1 = terra.wallets.test2;
const terra_user_2 = terra.wallets.test3;
const terra_user_3 = terra.wallets.test4;
const terra_user_4 = terra.wallets.test5;

let whale_token_address: string;
let airdrop_contract_address: string;

//----------------------------------------------------------------------------------------
// Setup : Test
//----------------------------------------------------------------------------------------

async function setupTest() {

    let whale_token_config = { "name": "WHALE",
                            "symbol": "WHALE",
                            "decimals": 6,
                            "initial_balances": [ {"address":deployer.key.accAddress, "amount":"100000000000000"}], 
                            "mint": { "minter":deployer.key.accAddress, "cap":"100000000000000"}
                           }
    whale_token_address = await deployContract(terra, deployer, join(ARTIFACTS_PATH, 'cw20_token.wasm'),  whale_token_config )
    console.log(chalk.green(`$WHALE deployed successfully, address : ${chalk.cyan(whale_token_address)}`));

    const init_timestamp = parseInt((Date.now()/1000).toFixed(0))
    const till_timestamp = init_timestamp + (86400 * 30)

    let airdrop_config = { "owner":  deployer.key.accAddress,
                         "whale_token_address": whale_token_address,
                         "merkle_roots": [],
                         "from_timestamp": init_timestamp, 
                         "till_timestamp": till_timestamp, 
                        } 
    
    airdrop_contract_address = await deployContract(terra, deployer, join(ARTIFACTS_PATH, 'whale_airdrop.wasm'),  airdrop_config )    
    const airdropConfigResponse = await getAirdropConfig(terra, airdrop_contract_address);
      expect(airdropConfigResponse).to.deep.equal({
        whale_token_address: whale_token_address,
        owner: deployer.key.accAddress,
        merkle_roots: [],
        from_timestamp: init_timestamp,
        till_timestamp: till_timestamp
      });
    console.log(chalk.green(`Airdrop Contract deployed successfully, address : ${chalk.cyan(airdrop_contract_address)}`));

    var contract_whale_balance_before_transfer = await getCW20Balance(terra, whale_token_address, airdrop_contract_address);
    var deployer_whale_balance_before_transfer = await getCW20Balance(terra, whale_token_address, deployer.key.accAddress);

    await transferCW20Tokens(terra, deployer, whale_token_address, airdrop_contract_address, 2500000 * 10**6 );

    var contract_whale_balance_after_transfer = await getCW20Balance(terra, whale_token_address, airdrop_contract_address);
    var deployer_whale_balance_after_transfer = await getCW20Balance(terra, whale_token_address, deployer.key.accAddress);

    expect(Number(contract_whale_balance_after_transfer) - Number(contract_whale_balance_before_transfer)).to.equal(2500000 * 10**6);
    expect(Number(deployer_whale_balance_before_transfer) - Number(deployer_whale_balance_after_transfer)).to.equal(2500000 * 10**6);
}

//----------------------------------------------------------------------------------------
// (ADMIN FUNCTION) Update Config : Test
//----------------------------------------------------------------------------------------

async function testUpdateConfig(merkle_roots: [string]) {
    process.stdout.write("Should update config info correctly... ");
    
    const init_timestamp = parseInt((Date.now()/1000).toFixed(0))
    const till_timestamp = init_timestamp + (86400 * 30)

    await updateAirdropConfig(terra, deployer, airdrop_contract_address,{ "update_config" : {  "new_config" : {
                                                                                                  "merkle_roots": merkle_roots,
                                                                                                  "from_timestamp": init_timestamp, 
                                                                                                  "till_timestamp": till_timestamp,  
                                                                                                  }
                                                                                             }                                                                                                                      
                                                                        });

    const airdropConfigResponse = await getAirdropConfig(terra, airdrop_contract_address);
    expect(airdropConfigResponse).to.deep.equal({ whale_token_address: whale_token_address,
                                                  owner: deployer.key.accAddress,
                                                  merkle_roots: merkle_roots,
                                                  from_timestamp: init_timestamp,
                                                  till_timestamp: till_timestamp,
                                                });
    console.log(chalk.green("\nMerkle roots updated successfully"));                                
}

//----------------------------------------------------------------------------------------
// Airdrop Claim By User : Test
//----------------------------------------------------------------------------------------

async function testClaimAirdrop(claimeeWallet:Wallet, amountClaimed:number, merkle_proof: any, root_index: number ) {
    process.stdout.write( `Should process claim for user  ${chalk.cyan(claimeeWallet.key.accAddress)} correctly... `);

   let is_claimed_before = await isAirdropClaimed(terra, airdrop_contract_address, claimeeWallet.key.accAddress);
   expect( is_claimed_before ).to.deep.equal( { is_claimed: false } );

    var contract_whale_balance_before_claim = await getCW20Balance(terra, whale_token_address, airdrop_contract_address);
    var user_whale_balance_before_claim = await getCW20Balance(terra, whale_token_address, claimeeWallet.key.accAddress);

    await claimAirdrop(terra,claimeeWallet, airdrop_contract_address,amountClaimed,merkle_proof,root_index);

    var contract_whale_balance_after_claim = await getCW20Balance(terra, whale_token_address, airdrop_contract_address);
    var user_whale_balance_after_claim = await getCW20Balance(terra, whale_token_address, claimeeWallet.key.accAddress);

    let is_claimed_after = await isAirdropClaimed(terra, airdrop_contract_address, claimeeWallet.key.accAddress);
    expect( is_claimed_after ).to.deep.equal( { is_claimed: true } );
 
    expect(Number(contract_whale_balance_before_claim) - Number(contract_whale_balance_after_claim)).to.equal(amountClaimed);
    expect(Number(user_whale_balance_after_claim) - Number(user_whale_balance_before_claim)).to.equal(amountClaimed);


    console.log(chalk.green( `\nClaim by user ${chalk.cyan(claimeeWallet.key.accAddress)} processed successfully` ));                                
}


//----------------------------------------------------------------------------------------
// (ADMIN FUNCTION) Transfer WHALE Tokens : Test
//----------------------------------------------------------------------------------------

async function testTransferUnclaimedWhaleByAdmin(recepient_address:string, amountToTransfer:number) {
    process.stdout.write("Should transfer WHALE from the Airdrop Contract correctly... ");
    
    var contract_whale_balance_before_claim = await getCW20Balance(terra, whale_token_address, airdrop_contract_address);
    var recepient_whale_balance_before_claim = await getCW20Balance(terra, whale_token_address, recepient_address );

    await transferUnclaimedWhaleFromAirdropContract(terra,deployer, airdrop_contract_address, recepient_address, amountToTransfer);

    var contract_whale_balance_after_claim = await getCW20Balance(terra, whale_token_address, airdrop_contract_address);
    var recepientWallet_whale_balance_after_claim = await getCW20Balance(terra, whale_token_address, recepient_address );

    expect(Number(contract_whale_balance_before_claim) - Number(contract_whale_balance_after_claim)).to.equal(amountToTransfer);
    expect(Number(recepientWallet_whale_balance_after_claim) - Number(recepient_whale_balance_before_claim)).to.equal(amountToTransfer);

    console.log(chalk.green("\nTransfer of WHALE tokens by the deployer with admin privileges processed successfully"));                                
}





//----------------------------------------------------------------------------------------
// Main
//----------------------------------------------------------------------------------------

(async () => {
    console.log(chalk.yellow("\n Airdrop Test: Info"));
  
    const toHexString = (bytes: Uint8Array) => bytes.reduce((str:string, byte:any) => str + byte.toString(16).padStart(2, '0'), '');

    console.log(`Deployer ::  ${chalk.cyan(deployer.key.accAddress)}`);

    console.log(`${chalk.cyan(terra_user_1.key.accAddress)} as Airdrop clamiee (terra) #1`);
    console.log(`${chalk.cyan(terra_user_2.key.accAddress)} as Airdrop clamiee (terra) #2`);
    console.log(`${chalk.cyan(terra_user_3.key.accAddress)} as Airdrop clamiee (terra) #3`);
    console.log(`${chalk.cyan(terra_user_4.key.accAddress)} as Airdrop clamiee (terra) #4`);

    // Deploy the contracts
    console.log(chalk.yellow("\nAirdrop Test: Setup"));
    await setupTest();

    // UpdateConfig :: Test
    console.log(chalk.yellow("\nTest: Update Configuration"));
    let terra_claimees_data = [ {"address":terra_user_1.key.accAddress, "amount": (250 * 10**6).toString()  },
                                {"address":terra_user_2.key.accAddress, "amount": (1).toString()  },
                                {"address":terra_user_3.key.accAddress, "amount": (71000 * 10**6).toString()  },
                                {"address":terra_user_4.key.accAddress, "amount": ( 10**6).toString()  },
                              ]
    let merkle_tree_terra = new Terra_Merkle_Tree(terra_claimees_data);
    let terra_tree_root = merkle_tree_terra.getMerkleRoot();
    await testUpdateConfig( [terra_tree_root] );

    // TransferWhaleTokens :: Test 
    console.log(chalk.yellow("\nTest: Transfer WHALE Tokens by Admin : "));
    await testTransferUnclaimedWhaleByAdmin(terra.wallets.test5.key.accAddress, 41000 * 10**6);

    // AirdropCLaim :: Test #1
    console.log(chalk.yellow("\nTest #1: Airdrop Claim By Terra user : " +  chalk.cyan(terra_user_1.key.accAddress)  ));
    let merkle_proof_for_terra_user_1 = merkle_tree_terra.getMerkleProof( {"address":terra_user_1.key.accAddress, "amount": (250 * 10**6).toString()  } );
    await testClaimAirdrop(terra_user_1, Number(terra_claimees_data[0]["amount"]), merkle_proof_for_terra_user_1, 0 )

    // AirdropCLaim :: Test #2
    console.log(chalk.yellow("\nTest #2: Airdrop Claim By Terra user : " + chalk.cyan(terra_user_2.key.accAddress) ));
    let merkle_proof_for_terra_user_2 = merkle_tree_terra.getMerkleProof( {"address":terra_user_2.key.accAddress, "amount": (1).toString()} );
    await testClaimAirdrop(terra_user_2, Number(terra_claimees_data[1]["amount"]), merkle_proof_for_terra_user_2, 0 )
    
    // AirdropCLaim :: Test #3
    console.log(chalk.yellow("\nTest #3: Airdrop Claim By Terra user : " + chalk.cyan(terra_user_3.key.accAddress) ));
    let merkle_proof_for_terra_user_3 = merkle_tree_terra.getMerkleProof( {"address":terra_user_3.key.accAddress, "amount": (71000 * 10**6).toString()} );
    await testClaimAirdrop(terra_user_3, Number(terra_claimees_data[2]["amount"]), merkle_proof_for_terra_user_3, 0 )

    // AirdropCLaim :: Test #4
    console.log(chalk.yellow("\nTest #4: Airdrop Claim By Terra user : " + chalk.cyan(terra_user_4.key.accAddress) ));
    let merkle_proof_for_terra_user_4 = merkle_tree_terra.getMerkleProof( {"address":terra_user_4.key.accAddress, "amount": ( 10**6).toString()} );
    await testClaimAirdrop(terra_user_4, Number(terra_claimees_data[3]["amount"]), merkle_proof_for_terra_user_4, 0 )

    console.log("");
  })();

