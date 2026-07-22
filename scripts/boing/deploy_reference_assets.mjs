#!/usr/bin/env node
/**
 * Deploy PudgyMon reference NFT collection + Saga Token fungible on Boing L1.
 *
 * Prerequisites:
 *   - boing-node RPC (default http://127.0.0.1:8545)
 *   - Local boing-sdk (BOING_SDK_PATH or sibling checkout)
 *   - Funded deployer: BOING_SECRET_HEX in scripts/boing/.env
 *
 * Usage (from repo root):
 *   node scripts/boing/deploy_reference_assets.mjs
 *
 * Writes AccountIds into data/boing/contracts.json
 */

import { readFileSync, writeFileSync, existsSync } from "node:fs";
import { createRequire } from "node:module";
import { dirname, resolve } from "node:path";
import { fileURLToPath } from "node:url";
import { pathToFileURL } from "node:url";

const __dirname = dirname(fileURLToPath(import.meta.url));
const repoRoot = resolve(__dirname, "../..");

function loadDotEnv(path) {
  if (!existsSync(path)) return;
  for (const line of readFileSync(path, "utf8").split(/\r?\n/)) {
    const t = line.trim();
    if (!t || t.startsWith("#") || !t.includes("=")) continue;
    const i = t.indexOf("=");
    const k = t.slice(0, i).trim();
    let v = t.slice(i + 1).trim();
    if (
      (v.startsWith('"') && v.endsWith('"')) ||
      (v.startsWith("'") && v.endsWith("'"))
    ) {
      v = v.slice(1, -1);
    }
    if (!(k in process.env)) process.env[k] = v;
  }
}

loadDotEnv(resolve(__dirname, ".env"));

const rpc = process.env.BOING_RPC_URL || "http://127.0.0.1:8545";
const secretHex = (process.env.BOING_SECRET_HEX || "").trim();
const autoFaucet =
  process.env.BOING_AUTO_FAUCET_REQUEST === "1" ||
  process.env.BOING_AUTO_FAUCET_REQUEST === "true";

function resolveSdkEntry() {
  const candidates = [
    process.env.BOING_SDK_PATH,
    resolve(
      "C:/Users/chiku/Desktop/vibe-code/boing.network/boing-sdk/dist/index.js"
    ),
    resolve(repoRoot, "../boing.network/boing-sdk/dist/index.js"),
    resolve(repoRoot, "../../Desktop/vibe-code/boing.network/boing-sdk/dist/index.js"),
  ].filter(Boolean);
  for (const p of candidates) {
    if (existsSync(p)) return p;
  }
  // npm-linked package
  try {
    const req = createRequire(import.meta.url);
    return req.resolve("boing-sdk");
  } catch {
    return null;
  }
}

function hexToBytes(hex) {
  const h = hex.startsWith("0x") ? hex.slice(2) : hex;
  if (h.length % 2) throw new Error("odd hex length");
  const out = new Uint8Array(h.length / 2);
  for (let i = 0; i < out.length; i++) {
    out[i] = parseInt(h.slice(i * 2, i * 2 + 2), 16);
  }
  return out;
}

function sleep(ms) {
  return new Promise((r) => setTimeout(r, ms));
}

async function main() {
  if (!secretHex || !/^0x[0-9a-fA-F]{64}$/.test(secretHex)) {
    console.error(
      "Set BOING_SECRET_HEX=0x…64 hex in scripts/boing/.env (32-byte Ed25519 seed)."
    );
    process.exit(1);
  }

  const sdkPath = resolveSdkEntry();
  if (!sdkPath) {
    console.error(
      "boing-sdk not found. Set BOING_SDK_PATH to boing-sdk/dist/index.js"
    );
    process.exit(1);
  }

  const sdk = await import(pathToFileURL(sdkPath).href);
  const {
    createClient,
    senderHexFromSecretKey,
    fetchNextNonce,
    predictNonceDerivedContractAddress,
    submitDeployWithPurposeFlow,
    buildReferenceFungibleDeployMetaTx,
    buildReferenceNftCollectionDeployMetaTx,
    hexToBytes: sdkHexToBytes,
    explainBoingRpcError,
  } = sdk;

  const toBytes = sdkHexToBytes || hexToBytes;
  const secretKey32 = toBytes(secretHex);
  const client = createClient(rpc);
  const senderHex = await senderHexFromSecretKey(secretKey32);

  console.log("RPC", rpc);
  console.log("Deployer", senderHex);

  const info = await client.getNetworkInfo();
  console.log(
    "Network",
    info?.chain_id ?? info?.chainId,
    info?.chain_name ?? ""
  );

  async function accountSnap() {
    const acct = await client.getAccount(senderHex);
    const height = await client.request("boing_chainHeight", []);
    return {
      height: Number(height ?? 0),
      nonce: BigInt(acct?.nonce ?? 0),
      balance: String(acct?.balance ?? "0"),
    };
  }

  async function waitForTipProgress(label, minHeight, timeoutMs = 45000) {
    const start = Date.now();
    let last = await accountSnap();
    while (Date.now() - start < timeoutMs) {
      if (last.height > minHeight) return last;
      await sleep(1000);
      last = await accountSnap();
    }
    throw new Error(
      `${label}: tip stuck at height ${last.height} (wanted > ${minHeight})`
    );
  }

  async function waitForNonce(label, beforeNonce, timeoutMs = 90000) {
    const start = Date.now();
    while (Date.now() - start < timeoutMs) {
      const snap = await accountSnap();
      if (snap.nonce > beforeNonce) return snap;
      await sleep(1000);
    }
    const snap = await accountSnap();
    throw new Error(
      `${label}: deploy not included (nonce still ${snap.nonce}, height ${snap.height})`
    );
  }

  {
    const tip = await accountSnap();
    console.log("Tip", tip);
  }

  // Tip only advances when mempool txs are included (no empty blocks).
  async function waitFunded(label, before, timeoutMs = 90000) {
    const start = Date.now();
    while (Date.now() - start < timeoutMs) {
      const snap = await accountSnap();
      if (
        snap.height > before.height ||
        BigInt(snap.balance) > BigInt(before.balance)
      ) {
        return snap;
      }
      await sleep(1000);
    }
    const snap = await accountSnap();
    throw new Error(
      `${label}: funding not included (height ${snap.height}, balance ${snap.balance})`
    );
  }

  if (autoFaucet && typeof client.faucetRequest === "function") {
    try {
      const before = await accountSnap();
      const f = await client.faucetRequest(senderHex);
      console.log("Faucet", f?.ok ? "ok" : f);
      const after = await waitFunded("post-faucet", before);
      console.log("Funded", after);
    } catch (e) {
      console.warn(
        "Faucet skipped/failed:",
        explainBoingRpcError?.(e) || e.message || e
      );
    }
  } else if (autoFaucet) {
    try {
      const before = await accountSnap();
      await client.request("boing_faucetRequest", [senderHex]);
      console.log("Faucet ok (raw)");
      const after = await waitFunded("post-faucet", before);
      console.log("Funded", after);
    } catch (e) {
      console.warn(
        "Faucet skipped/failed:",
        explainBoingRpcError?.(e) || e.message || e
      );
    }
  }

  // Reference deploys charge ~200_000 gas each (fee = gas × GAS_PRICE=1).
  const MIN_BALANCE_PER_DEPLOY = 200_000n;
  async function ensureDeployBalance(label) {
    const snap = await accountSnap();
    if (BigInt(snap.balance) < MIN_BALANCE_PER_DEPLOY) {
      throw new Error(
        `${label}: balance ${snap.balance} < ${MIN_BALANCE_PER_DEPLOY} deploy fee. ` +
          `Fund the deployer (single faucet dispense is only 50_000).`
      );
    }
  }

  async function deployMeta(meta, label) {
    await ensureDeployBalance(label);
    const bytecodeHex = meta.bytecode;
    const purpose = meta.purpose_category || "token";
    const nonce = await fetchNextNonce(client, senderHex);
    const nonceBefore = BigInt(nonce);
    const predicted = predictNonceDerivedContractAddress(senderHex, nonce);
    console.log(`Deploying ${label}… nonce=${nonce} predicted=${predicted}`);
    const result = await submitDeployWithPurposeFlow({
      client,
      secretKey32,
      senderHex,
      bytecode: toBytes(bytecodeHex),
      purposeCategory: purpose,
      descriptionHash: meta.description_hash
        ? toBytes(meta.description_hash)
        : null,
    });
    console.log(`  submit ${result.tx_hash} (attempts=${result.attempts})`);
    // Mempool admit often returns tx_hash "ok" — only trust account nonce advance.
    const included = await waitForNonce(label, nonceBefore);
    console.log(
      `  included height=${included.height} nonce=${included.nonce} account=${predicted}`
    );
    return {
      account: predicted,
      tx_hash: result.tx_hash,
      predicted,
    };
  }

  const fungibleMeta = buildReferenceFungibleDeployMetaTx({
    assetName: "Saga Token",
    assetSymbol: "SAGA",
  });
  const fungible = await deployMeta(fungibleMeta, "Saga Token (SAGA)");

  const nftMeta = buildReferenceNftCollectionDeployMetaTx({
    collectionName: "PudgyMon Skins",
    collectionSymbol: "PUDGY",
  });
  const nft = await deployMeta(nftMeta, "PudgyMon Skins (PUDGY)");

  if (fungible.account === nft.account) {
    throw new Error(
      `Deploy collision: fungible and NFT share ${fungible.account} (nonce did not advance between deploys).`
    );
  }

  const out = {
    rpc_url: rpc,
    chain_id: info?.chain_id ?? 6913,
    deployer: senderHex,
    fungible_name: "Saga Token",
    fungible_symbol: "SAGA",
    nft_collection_name: "PudgyMon Skins",
    nft_collection_symbol: "PUDGY",
    nft_collection: nft.account,
    fungible_token: fungible.account,
    deploy: {
      fungible_tx: fungible.tx_hash,
      nft_tx: nft.tx_hash,
    },
    notes:
      "Deployed by scripts/boing/deploy_reference_assets.mjs — NFT skins + Saga Token fungible.",
  };

  const outPath = resolve(repoRoot, "data/boing/contracts.json");
  writeFileSync(outPath, JSON.stringify(out, null, 2) + "\n");
  console.log("Wrote", outPath);
  console.log(JSON.stringify(out, null, 2));
}

main().catch((e) => {
  console.error(e?.stack || e);
  process.exit(1);
});
