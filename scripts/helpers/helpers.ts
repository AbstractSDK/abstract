import {
  Coin,
  CreateTxOptions,
  isTxError,
  LCDClient,
  LocalTerra,
  MnemonicKey,
  Msg,
  MsgExecuteContract,
  MsgInstantiateContract,
  MsgMigrateContract,
  MsgStoreCode,
  StdFee,
  Wallet
} from '@terra-money/terra.js';
import { readFileSync } from 'fs';

// Tequila lcd is load balanced, so txs can't be sent too fast, otherwise account sequence queries
// may resolve an older state depending on which lcd you end up with. Generally 1000 ms is is enough
// for all nodes to sync up.
let TIMEOUT = 1000

export function setTimeoutDuration(t: number) {
  TIMEOUT = t
}

export function getTimeoutDuration() {
  return TIMEOUT
}

// LocalTerra doesn't estimate fees properly, so we set the fee in this environment sufficiently high to
// ensure all transactions succeed.
const LOCAL_TERRA_FEE = new StdFee(
  30000000,
  [new Coin('uusd', 45000000)]
)

export async function performTransaction( terra: LocalTerra | LCDClient, wallet: Wallet, msg: Msg) {
  let options: CreateTxOptions = {
    msgs: [msg],
    gasPrices: [new Coin("uusd", 0.15)]
  }

  if (terra instanceof LocalTerra) {
    options.fee = LOCAL_TERRA_FEE
  }

  const tx = await wallet.createAndSignTx(options);

  const result = await terra.tx.broadcast(tx);
  if (isTxError(result)) {
    throw new Error(
      `transaction failed. code: ${result.code}, codespace: ${result.codespace}, raw_log: ${result.raw_log}`
    );
  }
  await new Promise(resolve => setTimeout(resolve, TIMEOUT));
  return result
}

export async function createTransaction( terra: LocalTerra | LCDClient, wallet: Wallet, msg: Msg) {
  let options: CreateTxOptions = {
    msgs: [msg],
    gasPrices: [new Coin("uusd", 0.15)]
  }

  if (terra instanceof LocalTerra) {
    options.fee = LOCAL_TERRA_FEE
  }

  return await wallet.createTx(options)
}

export async function uploadContract( terra: LocalTerra | LCDClient, wallet: Wallet, filepath: string) {
  const contract = readFileSync(filepath, 'base64');
  const uploadMsg = new MsgStoreCode(wallet.key.accAddress, contract);
  let result = await performTransaction(terra, wallet, uploadMsg);
  return Number(result.logs[0].eventsByType.store_code.code_id[0]) // code_id
}

export async function instantiateContract( terra: LocalTerra | LCDClient, wallet: Wallet, codeId: number, msg: object) {
  const instantiateMsg = new MsgInstantiateContract(wallet.key.accAddress, undefined , codeId, msg, undefined);
  let result = await performTransaction(terra, wallet, instantiateMsg)
  const attributes = result.logs[0].events[0].attributes
  return attributes[attributes.length - 1].value // contract address
}

export async function executeContract( terra: LocalTerra | LCDClient, wallet: Wallet, contractAddress: string, msg: object, coins?: any) {
  const executeMsg = new MsgExecuteContract(wallet.key.accAddress, contractAddress, msg, coins);
  return await performTransaction(terra, wallet, executeMsg);
}

export async function queryContract( terra: LocalTerra | LCDClient, contractAddress: string, query: object): Promise<any> {
  return await terra.wasm.contractQuery(contractAddress, query)
}

export async function deployContract( terra: LocalTerra | LCDClient, wallet: Wallet, filepath: string, initMsg: object) {
  const codeId = await uploadContract(terra, wallet, filepath);
  return await instantiateContract(terra, wallet, codeId, initMsg);
}

export async function migrate( terra: LocalTerra | LCDClient, wallet: Wallet, contractAddress: string, newCodeId: number) {
  const migrateMsg = new MsgMigrateContract(wallet.key.accAddress, contractAddress, newCodeId, {});
  return await performTransaction(terra, wallet, migrateMsg);
}

export function recover( terra: LocalTerra | LCDClient, mnemonic: string) {
  const mk = new MnemonicKey({ mnemonic: mnemonic });
  return terra.wallet(mk);
}

export function initialize(terra: LCDClient) {
  const mk = new MnemonicKey();

  console.log(`Account Address: ${mk.accAddress}`);
  console.log(`MnemonicKey: ${mk.mnemonic}`);

  return terra.wallet(mk);
}

export async function transferCW20Tokens(terra:LCDClient, wallet: Wallet, tokenAddress: string, recepient: string, amount: number) {
  let transfer_msg = {"transfer":{"recipient":recepient , "amount":amount.toString() }}
  let resp = await executeContract(terra, wallet, tokenAddress, transfer_msg );
}


// GET CW20 TOKEN BALANCE
export async function getCW20Balance(terra: LocalTerra | LCDClient, token_addr: string, user_address: string) {
  let curBalance = await terra.wasm.contractQuery<{ balance: string }>(token_addr, {"balance": {"address": user_address}} );
  return curBalance.balance
}

export function toEncodedBinary(object: any) {
  return Buffer.from(JSON.stringify(object)).toString('base64');
}
