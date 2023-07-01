const { execSync } = require("child_process");
execSync(
  `beaker wasm deploy osmosis-stargate --signer-account test1 --no-wasm-opt --admin signer --raw '{}'`,
  { stdio: "inherit" }
);

execSync(`beaker wasm ts-gen osmosis-stargate`, { stdio: "inherit" });
