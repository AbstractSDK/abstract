import  {Terra_Merkle_Tree}  from "./merkle_tree.js";
import airdropdata from "./airdrop_recepients.json";


const TERRA_MERKLE_ROOTS = 2;


// TERRA ECOSYSTEM AIRDROP :: RETURNS ROOTS OF THE MERKLE TREES FOR TERRA USERS
export async function getMerkleRootsForTerraUsers() { 
    let terra_merkle_roots = [];
    let n = TERRA_MERKLE_ROOTS;
  
    for (let i=0; i<n; i++ ) {
        let terra_data = prepareDataForMerkleTree(airdropdata.data , i * Math.round(airdropdata.data.length/n) , (i+1) * Math.round(airdropdata.data.length/n)  );
        let airdrop_tree = new Terra_Merkle_Tree(terra_data);
        let terra_merkle_root = airdrop_tree.getMerkleRoot();
        terra_merkle_roots.push(terra_merkle_root);            
    }
  
    return terra_merkle_roots;
  }
  

// TERRA ECOSYSTEM AIRDROP :: RETURNS MERKLE PROOF
export function get_Terra_MerkleProof( leaf: {address: string; amount: string;} ) {
    let Merkle_Trees = [];
    let n = TERRA_MERKLE_ROOTS;
  
    for (let i=0; i<n; i++ ) {
        let terra = prepareDataForMerkleTree(airdropdata.data , i * Math.round(airdropdata.data.length/n) , (i+1) * Math.round(airdropdata.data.length/n)  );
        let merkle_Tree = new Terra_Merkle_Tree(terra);
        Merkle_Trees.push(merkle_Tree);            
    }
  
    let proof = [];
    for (let i=0; i<Merkle_Trees.length; i++ ) {
        proof = Merkle_Trees[i].getMerkleProof( leaf );
        if (proof.length > 1) {
          return { "proof":proof, "root_index":i }; 
        }
    }
    return { "proof":null, "root_index":-1 }; 
  }  


// PREPARE DATA FOR THE MERKLE TREE
export function prepareDataForMerkleTree( data:(string | number)[][], str:number, end:number ) { 
    let dataArray = [];
    for ( let i=str; i < end; i++  ) {  
        let dataObj = JSON.parse( JSON.stringify(data[i]) );
        let ac = { "address":dataObj[0], "amount":dataObj[1].toString() };
        dataArray.push(ac);
    }
    return dataArray;
}