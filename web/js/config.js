// Production / Vercel: durable free Cloudflare Tunnel hostname.
// Origin is your local accounts stack (docker compose) while the named tunnel runs:
//   cloudflared tunnel --config %USERPROFILE%\.cloudflared\config.yml run boing-testnet-rpc
// Local file:// / localhost still default to http://127.0.0.1:8788 unless ?api= is set.
(() => {
  const host = location.hostname || "";
  if (host.includes("vercel.app") || host === "pudgymon.vercel.app") {
    window.PUGDYMON_API_BASE = "https://pudgymon-api.boing.network";
  }
})();
