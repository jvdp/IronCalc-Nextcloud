export async function model_load(fileId: string): Promise<Uint8Array> {
  return new Uint8Array(
    await (
      await fetch(`/index.php/apps/app_api/proxy/ironcalc/api/webdav/${fileId}`)
    ).arrayBuffer(),
  );
}
