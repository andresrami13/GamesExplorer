import { describe, expect, it } from "vitest";
import type { Game } from "../../domain/game";
import { artUrl } from "../../images/artUrl";
import { displayTitle, formatPlaytime, sortByTitle } from "./format";

const steam = (appId: number, title: string | null): Game => ({
  id: `steam:${appId}`,
  source: "steam",
  title,
  installed: true,
  installDir: null,
  playtimeMinutes: null,
  lastPlayed: null,
  launch: { kind: "steam", app_id: appId },
  color: null,
  metadata: null,
});

describe("formato de biblioteca", () => {
  it("usa un título de respaldo cuando la tienda aún no dio nombre", () => {
    expect(displayTitle(steam(440, null))).toBe("App 440");
  });

  it("ordena sin distinguir mayúsculas ni acentos y con números naturales", () => {
    const sorted = sortByTitle([steam(1, "Portal 2"), steam(2, "ábaco"), steam(3, "Portal 10"), steam(4, "Zelda")]);
    expect(sorted.map((g) => g.title)).toEqual(["ábaco", "Portal 2", "Portal 10", "Zelda"]);
  });

  it("formatea horas de juego", () => {
    expect(formatPlaytime(null)).toBeNull();
    expect(formatPlaytime(0)).toBeNull();
    expect(formatPlaytime(45)).toBe("45 min");
    expect(formatPlaytime(5160)).toBe("86 h");
  });

  it("construye la URL del protocolo de imágenes según la plataforma", () => {
    expect(artUrl(steam(570, "Dota 2"), "capsule", true)).toBe("http://img.localhost/steam/570/capsule");
    expect(artUrl(steam(570, "Dota 2"), "hero", false)).toBe("img://localhost/steam/570/hero");
    const epic: Game = { ...steam(0, "Alan"), source: "epic", launch: { kind: "epic", namespace: "n", catalog_item_id: "c", app_name: "a" } };
    expect(artUrl(epic, "capsule", true)).toBeNull();
  });
});
