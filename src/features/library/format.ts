import type { Game } from "../../domain/game";

const collator = new Intl.Collator("es", { sensitivity: "base", numeric: true });

export function displayTitle(game: Game): string {
  if (game.title) return game.title;
  return game.launch.kind === "steam" ? `App ${game.launch.app_id}` : game.id;
}

export function sortByTitle(games: Game[]): Game[] {
  return [...games].sort((a, b) => collator.compare(displayTitle(a), displayTitle(b)));
}

/** "86 h", "45 min" o `null` si no hay dato o nunca se jugó. */
export function formatPlaytime(minutes: number | null): string | null {
  if (!minutes) return null;
  return minutes < 60 ? `${minutes} min` : `${Math.round(minutes / 60)} h`;
}
