import { describe, expect, it } from "vitest";
import type { Game } from "../../domain/game";
import { artUrl } from "../../images/artUrl";
import { eyebrow, formatLastPlayed, formatPlaytime, visibleGames } from "./format";

const steam = (appId: number, title: string, lastPlayed: number | null = null): Game => ({
  id: `steam:${appId}`,
  source: "steam",
  title,
  installDir: null,
  playtimeMinutes: null,
  lastPlayed,
  launch: { kind: "steam", app_id: appId },
  color: null,
  metadata: null,
});

describe("lista visible", () => {
  const games = [steam(1, "Portal 2", 100), steam(2, "ábaco", 300), steam(3, "Portal 10"), steam(4, "Pokémon", 200)];

  it("ordena por nombre sin distinguir mayúsculas ni acentos y con números naturales", () => {
    expect(visibleGames(games, "installed", "").map((g) => g.title)).toEqual(["ábaco", "Pokémon", "Portal 2", "Portal 10"]);
  });

  it("ordena por última sesión, y lo nunca jugado al final", () => {
    expect(visibleGames(games, "recent", "").map((g) => g.title)).toEqual(["ábaco", "Pokémon", "Portal 2", "Portal 10"]);
  });

  it("busca sin acentos", () => {
    expect(visibleGames(games, "installed", " pokemon ").map((g) => g.title)).toEqual(["Pokémon"]);
  });
});

describe("formato", () => {
  it("formatea horas de juego", () => {
    expect(formatPlaytime(null)).toBeNull();
    expect(formatPlaytime(0)).toBeNull();
    expect(formatPlaytime(45)).toBe("45 min");
    expect(formatPlaytime(5160)).toBe("86 h");
  });

  it("formatea la última sesión en días naturales", () => {
    const now = new Date(2026, 10, 24, 1, 0);
    const at = (d: Date) => d.getTime() / 1000;
    expect(formatLastPlayed(null, now)).toBeNull();
    expect(formatLastPlayed(at(new Date(2026, 10, 24, 0, 30)), now)).toBe("Hoy");
    expect(formatLastPlayed(at(new Date(2026, 10, 23, 23, 59)), now)).toBe("Ayer");
    expect(formatLastPlayed(at(new Date(2026, 10, 19, 12)), now)).toBe("Hace 5 días");
  });

  it("compone géneros y año", () => {
    const game = { ...steam(1, "Hades II"), metadata: { description: null, genres: ["Acción", "Roguelike", "Indie"], year: 2025 } };
    expect(eyebrow(game)).toBe("Acción · Roguelike · 2025");
    expect(eyebrow(steam(2, "Sin datos"))).toBeNull();
  });

  it("construye la URL del protocolo de imágenes según la plataforma", () => {
    expect(artUrl(steam(570, "Dota 2"), "capsule", true)).toBe("http://img.localhost/steam/570/capsule");
    expect(artUrl(steam(570, "Dota 2"), "hero", false)).toBe("img://localhost/steam/570/hero");
    const epic: Game = { ...steam(0, "Alan"), source: "epic", launch: { kind: "epic", namespace: "n", catalog_item_id: "c", app_name: "a" } };
    expect(artUrl(epic, "capsule", true)).toBeNull();
  });
});
