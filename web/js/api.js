(() => {
  const DEFAULT_API = "http://127.0.0.1:8788";
  const TOKEN_KEY = "pudgymon_access_token";

  function apiBase() {
    try {
      const params = new URLSearchParams(window.location.search);
      const fromQuery = params.get("api");
      if (fromQuery) {
        localStorage.setItem("pudgymon_api_base", fromQuery);
        return fromQuery.replace(/\/$/, "");
      }
    } catch (_) {}
    return (
      localStorage.getItem("pudgymon_api_base") ||
      window.PUDGYMON_API_BASE ||
      DEFAULT_API
    ).replace(/\/$/, "");
  }

  function getToken() {
    return localStorage.getItem(TOKEN_KEY) || "";
  }

  function setToken(token) {
    if (token) localStorage.setItem(TOKEN_KEY, token);
    else localStorage.removeItem(TOKEN_KEY);
  }

  async function request(path, options = {}) {
    const headers = {
      "Content-Type": "application/json",
      ...(options.headers || {}),
    };
    const token = getToken();
    if (token && !headers.Authorization) {
      headers.Authorization = `Bearer ${token}`;
    }
    const res = await fetch(`${apiBase()}${path}`, {
      ...options,
      headers,
    });
    const text = await res.text();
    let data = null;
    try {
      data = text ? JSON.parse(text) : null;
    } catch (_) {
      data = { error: text || res.statusText };
    }
    if (!res.ok) {
      const err = new Error((data && data.error) || res.statusText || "request failed");
      err.status = res.status;
      err.data = data;
      throw err;
    }
    return data;
  }

  async function signup(email, password, display_name) {
    const data = await request("/v1/auth/signup", {
      method: "POST",
      body: JSON.stringify({ email, password, display_name }),
    });
    setToken(data.access_token);
    return data;
  }

  async function login(email, password) {
    const data = await request("/v1/auth/login", {
      method: "POST",
      body: JSON.stringify({ email, password }),
    });
    setToken(data.access_token);
    return data;
  }

  async function me() {
    return request("/v1/me");
  }

  async function patchMe(body) {
    return request("/v1/me", {
      method: "PATCH",
      body: JSON.stringify(body),
    });
  }

  function logout() {
    setToken("");
  }

  function downloadGameToken(token) {
    const blob = new Blob([token || getToken()], { type: "text/plain" });
    const a = document.createElement("a");
    a.href = URL.createObjectURL(blob);
    a.download = "pending_token.txt";
    a.click();
    URL.revokeObjectURL(a.href);
  }

  window.PudgyMonApi = {
    apiBase,
    getToken,
    setToken,
    signup,
    login,
    me,
    patchMe,
    logout,
    downloadGameToken,
  };
})();
