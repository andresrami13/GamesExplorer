const MAX_ENTRIES = 300;
const loaded = new Map<string, Promise<boolean>>();

/**
 * Descarga y decodifica una imagen antes de mostrarla, para que el fundido empiece
 * con la imagen ya lista. Resuelve `false` si no existe.
 */
export function preload(url: string | null): Promise<boolean> {
  if (!url) return Promise.resolve(false);
  const known = loaded.get(url);
  if (known) return known;
  const img = new Image();
  img.src = url;
  const result = img.decode().then(
    () => true,
    () => false,
  );
  loaded.set(url, result);
  if (loaded.size > MAX_ENTRIES) loaded.delete(loaded.keys().next().value!);
  return result;
}
