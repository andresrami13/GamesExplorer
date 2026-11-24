import { create } from "zustand";
import type { View } from "../features/library/format";

interface UiState {
  view: View;
  query: string;
  /** Por id y no por posición: la selección sobrevive a búsquedas y cambios de orden. */
  selectedId: string | null;
  setView: (view: View) => void;
  setQuery: (query: string) => void;
  select: (id: string) => void;
}

export const useUi = create<UiState>((set) => ({
  view: "installed",
  query: "",
  selectedId: null,
  setView: (view) => set({ view }),
  setQuery: (query) => set({ query }),
  select: (selectedId) => set({ selectedId }),
}));
