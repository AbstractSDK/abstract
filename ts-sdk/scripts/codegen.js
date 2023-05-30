const codegen = require("@abstract-money/ts-codegen").default;
const path = require("path");
const fs = require("fs");

const pkgRoot = path.join(__dirname, "..");
const contractsDir = path.join(pkgRoot, "..", "..");

const contract = {
  name: 'Template',
  dir:  path.join(pkgRoot, "..", "..", "schema")
}

console.log(contract)

// Leaving unused, we need to identify whether they're in a workspace eventually
const contracts = fs
  .readdirSync(contractsDir, { withFileTypes: true })
  .filter((c) => c.isDirectory())
  .map((c) => ({
    name: c.name,
    dir: path.join(contractsDir, c.name),
  }));


const outPath = path.join(pkgRoot, "src", "contracts");
fs.rmSync(outPath, { recursive: true, force: true });

codegen({
  contracts: [contract],
  outPath,
  options: {
    bundle: {
      bundleFile: "index.ts",
      scope: "contracts",
    },
     messageComposer: {
       enabled: true,
     },
     msgBuilder: {
       enabled: true,
     },
     abstractApp: {
       enabled: true,
       clientPrefix: '',
     },
     types: {
       aliasExecuteMsg: true,
       // aliasQueryMsg: true,
     }
  },
}).then(() => {
  console.log("✨ Typescript code is generated successfully!");
});
