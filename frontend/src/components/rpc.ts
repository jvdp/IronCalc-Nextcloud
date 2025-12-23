export async function get_webdav(fileId: string): Promise<Uint8Array> {
  return new Uint8Array(
    await (await fetch(`/api/webdav/${fileId}`)).arrayBuffer(),
  );
}
