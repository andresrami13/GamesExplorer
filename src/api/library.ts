import { invoke } from "@tauri-apps/api/core";
import { listen, type UnlistenFn } from "@tauri-apps/api/event";
import type { Game, Library } from "../domain/game";

export const getCachedLibrary = () => invoke<Game[]>("get_cached_library");
export const refreshLibrary = () => invoke<Library>("refresh_library");
export const launchGame = (id: string) => invoke<void>("launch_game", { id });
export const openStorePage = (id: string) => invoke<void>("open_store_page", { id });

export const onGameUpdated = (handler: (game: Game) => void): Promise<UnlistenFn> =>
  listen<Game>("game-updated", (e) => handler(e.payload));

export const onGameRemoved = (handler: (id: string) => void): Promise<UnlistenFn> =>
  listen<string>("game-removed", (e) => handler(e.payload));
