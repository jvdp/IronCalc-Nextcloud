export async function model_load(
  fileId: string,
  lang: string,
  tz: string,
): Promise<Uint8Array> {
  const params = new URLSearchParams({ lang, tz });
  return new Uint8Array(
    await (
      await fetch(
        `/index.php/apps/app_api/proxy/ironcalc/api/webdav/${fileId}?${params}`,
      )
    ).arrayBuffer(),
  );
}
