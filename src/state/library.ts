import { create } from "zustand";
import * as api from "../api/library";
import type { Game } from "../domain/game";

type Status = "loading" | "ready" | "error";

interface LibraryState {
  games: Game[];
  status: Status;
  /** Errores por plataforma, o el motivo de que falle la carga entera. */
  errors: string[];
  start: () => Promise<void>;
}

let started = false;

export const useLibrary = create<LibraryState>((set) => ({
  games: [],
  status: "loading",
  errors: [],

  // Pinta la instantánea al instante y después la sustituye por la lectura de disco;
  // los juegos se van completando con los eventos del enriquecedor.
  start: async () => {
    if (started) return; // StrictMode monta dos veces en desarrollo.
    started = true;

    await api.onGameUpdated((game) => set((s) => ({ games: s.games.map((g) => (g.id === game.id ? game : g)) })));
    await api.onGameRemoved((id) => set((s) => ({ games: s.games.filter((g) => g.id !== id) })));

    const cached = await api.getCachedLibrary();
    if (cached.length > 0) set({ games: cached, status: "ready" });

    try {
      const { games, errors } = await api.refreshLibrary();
      set({ games, errors, status: "ready" });
    } catch (e) {
      set((s) => ({ status: s.games.length > 0 ? "ready" : "error", errors: [String(e)] }));
    }
  },
}));
