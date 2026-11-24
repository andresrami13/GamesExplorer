import type { Game } from "../../domain/game";

export type View = "installed" | "recent";

const collator = new Intl.Collator("es", { sensitivity: "base", numeric: true });
const DAY_MS = 86_400_000;

/** Minúsculas y sin acentos, para buscar "pokemon" y encontrar "Pokémon". */
const normalize = (s: string) => s.normalize("NFD").replace(/\p{Diacritic}/gu, "").toLowerCase();

export function visibleGames(games: Game[], view: View, query: string): Game[] {
  const q = normalize(query.trim());
  const found = q ? games.filter((g) => normalize(g.title).includes(q)) : [...games];
  return view === "recent"
    ? found.sort((a, b) => (b.lastPlayed ?? 0) - (a.lastPlayed ?? 0) || collator.compare(a.title, b.title))
    : found.sort((a, b) => collator.compare(a.title, b.title));
}

/** "86 h", "45 min" o `null` si no hay dato o nunca se jugó. */
export function formatPlaytime(minutes: number | null): string | null {
  if (!minutes) return null;
  return minutes < 60 ? `${minutes} min` : `${Math.round(minutes / 60)} h`;
}

/** "Hoy", "Ayer", "Hace 5 días" o la fecha. `lastPlayed` en segundos Unix. */
export function formatLastPlayed(lastPlayed: number | null, now = new Date()): string | null {
  if (!lastPlayed) return null;
  const played = new Date(lastPlayed * 1000);
  const startOfDay = (d: Date) => new Date(d.getFullYear(), d.getMonth(), d.getDate()).getTime();
  const days = Math.round((startOfDay(now) - startOfDay(played)) / DAY_MS);
  if (days <= 0) return "Hoy";
  if (days === 1) return "Ayer";
  if (days < 30) return `Hace ${days} días`;
  return played.toLocaleDateString("es", { day: "numeric", month: "short", year: "numeric" });
}

/** "ACCIÓN · ROGUELIKE · 2024" a partir de los metadatos de la tienda. */
export function eyebrow(game: Game): string | null {
  const parts = [...(game.metadata?.genres.slice(0, 2) ?? []), game.metadata?.year?.toString()].filter(Boolean);
  return parts.length ? parts.join(" · ") : null;
}
