(() => {
  const path = location.pathname.split("/").pop() || "index.html";
  document.querySelectorAll("[data-nav]").forEach((el) => {
    if (el.getAttribute("href") === path) {
      el.style.color = "var(--ink)";
    }
  });

  // Optional live API probe for footers that include #apiHealth.
  const healthEl = document.getElementById("apiHealth");
  if (!healthEl || !window.PugdyMonApi) return;
  const base = PugdyMonApi.apiBase();
  healthEl.textContent = `API ${base} · checking…`;
  fetch(`${base}/health`, { method: "GET" })
    .then((res) => {
      healthEl.textContent = res.ok
        ? `API ${base} · online`
        : `API ${base} · HTTP ${res.status}`;
    })
    .catch(() => {
      healthEl.textContent = `API ${base} · offline (start services/accounts)`;
    });
})();
