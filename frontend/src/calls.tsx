const API_BASE = "/index.php/apps/app_api/proxy/ironcalc/api/workbook";

export async function workbook_load(
  path: string,
  lang: string,
  tz: string,
): Promise<Uint8Array> {
  const params = new URLSearchParams({ path, lang, tz });
  const resp = await fetch(`${API_BASE}?${params}`);
  if (!resp.ok) {
    throw new Error("Failed to load workbook");
  }
  return new Uint8Array(await resp.arrayBuffer());
}

export async function workbook_rename(
  path: string,
  newName: string,
): Promise<void> {
  const params = new URLSearchParams({ path, name: newName });
  const resp = await fetch(`${API_BASE}/rename?${params}`, { method: "POST" });
  if (resp.status === 409) {
    throw new Error("A file with that name already exists");
  }
  if (!resp.ok) {
    throw new Error("Rename failed");
  }
}

export async function workbook_save(
  path: string,
  modelBytes: Uint8Array,
  lang: string,
): Promise<void> {
  const params = new URLSearchParams({ path, lang });
  const resp = await fetch(`${API_BASE}?${params}`, {
    method: "PUT",
    body: modelBytes.buffer as ArrayBuffer,
  });
  if (!resp.ok) {
    throw new Error("Save failed");
  }
}
