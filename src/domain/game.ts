// Espejo de `mis_juegos_core::game` tal como lo serializa serde.

export type Source = "steam" | "epic";

export type LaunchTarget =
  | { kind: "steam"; app_id: number }
  | { kind: "epic"; namespace: string; catalog_item_id: string; app_name: string };

export interface Metadata {
  description: string | null;
  genres: string[];
  year: number | null;
}

export interface Game {
  id: string;
  source: Source;
  title: string | null;
  installed: boolean;
  installDir: string | null;
  playtimeMinutes: number | null;
  /** Segundos Unix. */
  lastPlayed: number | null;
  launch: LaunchTarget;
  /** Color dominante de la portada, `#rrggbb`. */
  color: string | null;
  metadata: Metadata | null;
}

export interface Library {
  games: Game[];
  errors: string[];
}
