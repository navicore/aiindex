const BASE = '';

async function fetchJson(path) {
  const res = await fetch(`${BASE}${path}`);
  if (!res.ok) throw new Error(`${res.status} ${res.statusText}`);
  return res.json();
}

export function getIndex() {
  return fetchJson('/api/index');
}

export function getIndexHistory(limit = 100) {
  return fetchJson(`/api/index/history?limit=${limit}`);
}

export function getStocks() {
  return fetchJson('/api/stocks');
}

export function getStock(symbol) {
  return fetchJson(`/api/stocks/${symbol}`);
}

export function getSectors() {
  return fetchJson('/api/sectors');
}

export function getConfig() {
  return fetchJson('/api/config');
}
