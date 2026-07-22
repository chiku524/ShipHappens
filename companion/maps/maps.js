(() => {
  const packEl = document.getElementById("pack");
  const status = document.getElementById("status");
  const summary = document.getElementById("summary");
  const downloadBtn = document.getElementById("downloadBtn");
  let parsed = null;

  function show(msg, err) {
    status.hidden = false;
    status.textContent = msg;
    status.classList.toggle("err", !!err);
  }

  function validate(data) {
    if (!data || typeof data !== "object") throw new Error("Expected JSON object");
    const kind = data.kind || data.mode || "race";
    if (kind === "party_saga") {
      if (!data.race || !data.vibe || !data.shooter) {
        throw new Error("party_saga pack needs race, vibe, shooter");
      }
      if (!Array.isArray(data.race.gates) || data.race.gates.length < 2) {
        throw new Error("race needs ≥2 gates");
      }
      if (!Array.isArray(data.vibe.orbs) || data.vibe.orbs.length < 3) {
        throw new Error("vibe needs ≥3 orbs");
      }
    } else if (kind === "race") {
      if (!Array.isArray(data.gates) || data.gates.length < 2) {
        throw new Error("race needs ≥2 gates");
      }
    } else if (kind === "vibe") {
      if (!Array.isArray(data.orbs) || data.orbs.length < 3) {
        throw new Error("vibe needs ≥3 orbs");
      }
    } else if (kind === "shooter") {
      if (!Array.isArray(data.spawns) || data.spawns.length < 1) {
        throw new Error("shooter needs spawns");
      }
    } else {
      throw new Error(`unsupported kind/mode: ${kind}`);
    }
    return { kind, id: data.id || "imported", label: data.label || data.id || "Imported" };
  }

  document.getElementById("validateBtn").onclick = () => {
    try {
      parsed = JSON.parse(packEl.value);
      const meta = validate(parsed);
      summary.hidden = false;
      document.getElementById("outKind").textContent = meta.kind;
      document.getElementById("outId").textContent = meta.id;
      document.getElementById("outLabel").textContent = meta.label;
      downloadBtn.disabled = false;
      show("Looks good — download and drop into maps/ or maps/shares/.", false);
    } catch (e) {
      parsed = null;
      summary.hidden = true;
      downloadBtn.disabled = true;
      show(String(e.message || e), true);
    }
  };

  document.getElementById("fileBtn").onclick = () =>
    document.getElementById("fileInput").click();
  document.getElementById("fileInput").onchange = async (ev) => {
    const file = ev.target.files?.[0];
    if (!file) return;
    packEl.value = await file.text();
  };

  downloadBtn.onclick = () => {
    if (!parsed) return;
    const blob = new Blob([JSON.stringify(parsed, null, 2)], {
      type: "application/json",
    });
    const a = document.createElement("a");
    a.href = URL.createObjectURL(blob);
    a.download = `${parsed.id || "pudgymon_map"}.json`;
    a.click();
  };
})();
