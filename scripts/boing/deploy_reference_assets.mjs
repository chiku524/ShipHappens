#!/usr/bin/env node
/**
 * Deploy reference NFT collection + fungible token on Boing Network (testnet/local).
 *
 * Prerequisites:
 *   - boing-node RPC at BOING_RPC_URL (default http://127.0.0.1:8545)
 *   - boing-sdk available (clone Boing Network repo and npm link / path)
 *   - funded deployer key via BOING_DEPLOYER_* per boing-sdk docs
 *
 * Usage:
 *   node scripts/boing/deploy_reference_assets.mjs
 *
 * Then paste AccountIds into data/boing/contracts.json
 */

import { writeFileSync } from "node:fs";
import { resolve } from "node:path";

const rpc = process.env.BOING_RPC_URL || "http://127.0.0.1:8545";

async function main() {
  let sdk;
  try {
    sdk = await import("boing-sdk");
  } catch {
    console.error(
      "boing-sdk not found. From the Boing Network repo: npm install && npm run build in boing-sdk/\n" +
        "Then: npm link boing-sdk  (or NODE_PATH to that package)."
    );
    process.exit(1);
  }

  const client = sdk.createClient(rpc);
  const info = await client.getNetworkInfo?.() ?? await client.request("boing_getNetworkInfo", []);
  console.log("Connected:", info?.chain_id ?? info?.chainId ?? rpc);

  // Prefer SDK helpers when present; otherwise print manual checklist.
  if (typeof sdk.buildReferenceNftCollectionDeployMetaTx !== "function") {
    console.log(`
Manual deploy checklist (boing-sdk):
  1. buildReferenceNftCollectionDeployMetaTx({ assetName: 'PugdyMon Skins', assetSymbol: 'PUGDY', bytecodeHexOverride })
  2. preflightContractDeployMetaWithUi(client, tx)
  3. boing_sendTransaction via Express / submitDeployWithPurposeFlow
  4. Repeat for fungible: buildReferenceFungibleDeployMetaTx({ assetName: 'Party Coin', assetSymbol: 'PTY' })
  5. Write AccountIds to data/boing/contracts.json
`);
    const stub = {
      rpc_url: rpc,
      nft_collection: null,
      fungible_token: null,
      notes: "Fill after deploy",
    };
    const out = resolve("data/boing/contracts.json");
    writeFileSync(out, JSON.stringify(stub, null, 2) + "\n");
    console.log("Wrote stub", out);
    return;
  }

  console.log("SDK helpers present — wire deployer credentials and extend this script for full automation.");
}

main().catch((e) => {
  console.error(e);
  process.exit(1);
});
