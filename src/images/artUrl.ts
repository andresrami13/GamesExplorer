import type { Game } from "../domain/game";

export type ArtKind = "capsule" | "hero" | "logo" | "header";

const runningOnWindows = typeof navigator !== "undefined" && navigator.userAgent.includes("Windows");

/**
 * URL del protocolo `img` que sirve el backend. WebView2 (Windows) solo acepta
 * protocolos propios con la forma `http://<esquema>.localhost/`.
 * Devuelve `null` si la plataforma del juego no tiene arte disponible todavía.
 */
export function artUrl(game: Game, kind: ArtKind, windows = runningOnWindows): string | null {
  if (game.launch.kind !== "steam") return null;
  const path = `steam/${game.launch.app_id}/${kind}`;
  return windows ? `http://img.localhost/${path}` : `img://localhost/${path}`;
}
