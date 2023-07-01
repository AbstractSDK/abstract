const fs = require("fs");
const path = require("path");
const { SigningCosmWasmClient, Secp256k1HdWallet } = require("cosmwasm");
const { stringToPath } = require("@cosmjs/crypto");
const { OsmosisStargateContract } = require("osmosis-stargate-sdk");
const { OsmosisStargateClient } = OsmosisStargateContract;

jest.setTimeout(100000);

let contractAddr;
let client;
let osmosisStargateClient;

beforeAll(async () => {
  contractAddr = JSON.parse(
    fs.readFileSync(path.join(__dirname, "..", ".beaker", "state.local.json"))
  ).local["osmosis-stargate"].addresses.default;

  const walletConf = {
    prefix: "osmo",
    hdPaths: [stringToPath("m/44'/118'/0'/0/0")],
  };
  const mnemonic =
    "notice oak worry limit wrap speak medal online prefer cluster roof addict wrist behave treat actual wasp year salad speed social layer crew genius";
  const signer = await Secp256k1HdWallet.fromMnemonic(mnemonic, walletConf);

  const sender = (await signer.getAccounts())[0].address;

  const rpcEndpoint = "http://localhost:26657";

  client = await SigningCosmWasmClient.connectWithSigner(rpcEndpoint, signer, {
    gasPrice: "1uosmo",
  });

  osmosisStargateClient = new OsmosisStargateClient(
    client,
    sender,
    contractAddr
  );
});

test("tokenfactory createDenom with initial mint", async () => {
  const id = Math.random();
  const subdenom = `token-${id}`.replace("0.", "");
  const initialMint = "10000000000000";

  const createDenomReqUosmo = 10000000;
  const createPoolReqUosmo = 100000000;
  const poolUosmo = 1000000000;

  const initialPool = {
    swap_fee: "1", // floating point cause unpack error
    exit_fee: "1",
    pairing_denom: "uosmo",
    pool_assets: {
      new_token_amount: initialMint,
      new_token_weight: "1",
      pairing_token_amount: `${poolUosmo}`,
      pairing_token_weight: "1",
    },
  };

  // create denom and mint
  const res = await osmosisStargateClient.createDenom(
    { subdenom, initialMint, initialPool },
    "auto",
    undefined,
    [
      {
        denom: "uosmo",
        amount: `${createDenomReqUosmo + createPoolReqUosmo + poolUosmo}`,
      },
    ]
  );

  expect(getEventAttr(res, "create_denom", "new_token_denom")).toBe(
    `factory/${contractAddr}/${subdenom}`
  );
  expect(getEventAttr(res, "mint", "mint_to_address")).toBe(contractAddr);
  expect(getEventAttr(res, "mint", "amount")).toBe(
    `${initialMint}factory/${contractAddr}/${subdenom}`
  );

  const poolId = getEventAttr(res, "pool_created", "pool_id");

  // from submsg reply response
  expect(getEventAttr(res, "wasm", "pool_id")).toBe(poolId);

  const lpToken = `100000000000000000000gamm/pool/${poolId}`;
  expect(
    getEventAttr(res, "transfer").attributes.find(
      (attr) => attr.value === lpToken
    )
  ).toBeTruthy();
});

const getEventAttr = (res, eventType, key) => {
  const e = res.logs[0].events.find((e) => e.type === eventType);

  if (key !== undefined) {
    return e.attributes.find((a) => a.key === key).value;
  }

  return e;
};
