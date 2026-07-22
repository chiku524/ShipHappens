(() => {
  const els = {
    voucher: document.getElementById("voucher"),
    parseBtn: document.getElementById("parseBtn"),
    fileBtn: document.getElementById("fileBtn"),
    fileInput: document.getElementById("fileInput"),
    parseStatus: document.getElementById("parseStatus"),
    summary: document.getElementById("summary"),
    outSkin: document.getElementById("outSkin"),
    outAccount: document.getElementById("outAccount"),
    outPoints: document.getElementById("outPoints"),
    outNote: document.getElementById("outNote"),
    expressBtn: document.getElementById("expressBtn"),
    mintBtn: document.getElementById("mintBtn"),
    mintStatus: document.getElementById("mintStatus"),
  };

  let parsed = null;

  function showStatus(el, msg, isErr) {
    el.hidden = false;
    el.textContent = msg;
    el.classList.toggle("err", !!isErr);
  }

  function applyVoucher(data) {
    if (!data || typeof data !== "object") {
      throw new Error("Voucher must be a JSON object");
    }
    if (!data.account || !String(data.account).startsWith("0x")) {
      throw new Error("Missing account (0x + 64 hex)");
    }
    parsed = {
      skin_id: data.skin_id || "unknown",
      account: String(data.account),
      season_points: Number(data.season_points || 0),
      note: data.note || "",
    };
    els.outSkin.textContent = parsed.skin_id;
    els.outAccount.textContent = parsed.account;
    els.outPoints.textContent = String(parsed.season_points);
    els.outNote.textContent = parsed.note || "—";
    els.summary.hidden = false;
    els.mintBtn.disabled = false;
    localStorage.setItem("pudgymon_claim_voucher", JSON.stringify(parsed));
    showStatus(els.parseStatus, "Voucher parsed. Ready for Express mint path.", false);
  }

  els.parseBtn.addEventListener("click", () => {
    try {
      applyVoucher(JSON.parse(els.voucher.value));
    } catch (err) {
      showStatus(els.parseStatus, String(err.message || err), true);
      els.summary.hidden = true;
      els.mintBtn.disabled = true;
    }
  });

  els.fileBtn.addEventListener("click", () => els.fileInput.click());
  els.fileInput.addEventListener("change", async () => {
    const file = els.fileInput.files?.[0];
    if (!file) return;
    const text = await file.text();
    els.voucher.value = text;
    try {
      applyVoucher(JSON.parse(text));
    } catch (err) {
      showStatus(els.parseStatus, String(err.message || err), true);
    }
  });

  els.expressBtn.addEventListener("click", () => {
    window.open("https://boing.express", "_blank", "noopener");
  });

  els.mintBtn.addEventListener("click", async () => {
    if (!parsed) return;
    const provider = window.boing;
    if (!provider || typeof provider.request !== "function") {
      showStatus(
        els.mintStatus,
        "window.boing not found. Open this page from Boing Express (or install the extension), then retry. Voucher is saved in localStorage.",
        true
      );
      return;
    }
    try {
      // Placeholder intent — replace with collection contract_call once contracts.json is filled.
      const result = await provider.request({
        method: "boing_sendTransaction",
        params: [
          {
            kind: "pudgymon_claim_intent",
            skin_id: parsed.skin_id,
            account: parsed.account,
            season_points: parsed.season_points,
            note: "Build real mint meta-tx via boing-sdk against contracts.json",
          },
        ],
      });
      showStatus(els.mintStatus, `Express response: ${JSON.stringify(result)}`, false);
    } catch (err) {
      showStatus(els.mintStatus, `Express error: ${err.message || err}`, true);
    }
  });

  const saved = localStorage.getItem("pudgymon_claim_voucher");
  if (saved) {
    els.voucher.value = saved;
    try {
      applyVoucher(JSON.parse(saved));
    } catch {
      /* ignore corrupt cache */
    }
  }
})();
