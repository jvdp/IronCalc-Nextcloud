const API_BASE = "/index.php/apps/app_api/proxy/ironcalc/api/workbook";

export async function workbook_load(
  fileId: string,
  lang: string,
  tz: string,
  path?: string,
): Promise<Uint8Array> {
  const params = new URLSearchParams({ lang, tz });
  if (path) {
    params.set("path", path);
  }
  const resp = await fetch(`${API_BASE}/${fileId}?${params}`);
  if (!resp.ok) {
    throw new Error("Failed to load workbook");
  }
  return new Uint8Array(await resp.arrayBuffer());
}

export async function workbook_rename(
  fileId: string,
  newName: string,
): Promise<void> {
  const resp = await fetch(
    `${API_BASE}/${fileId}/rename?name=${encodeURIComponent(newName)}`,
    { method: "POST" },
  );
  if (resp.status === 409) {
    throw new Error("A file with that name already exists");
  }
  if (!resp.ok) {
    throw new Error("Rename failed");
  }
}

export async function workbook_save(
  fileId: string,
  modelBytes: Uint8Array,
  lang: string,
  path?: string,
): Promise<number> {
  const params = new URLSearchParams({ lang });
  if (path) {
    params.set("path", path);
  }
  const resp = await fetch(`${API_BASE}/${fileId}?${params}`, {
    method: "PUT",
    body: modelBytes.buffer as ArrayBuffer,
  });
  if (!resp.ok) {
    throw new Error("Save failed");
  }
  const { fileId: newFileId } = await resp.json();
  return newFileId;
}
