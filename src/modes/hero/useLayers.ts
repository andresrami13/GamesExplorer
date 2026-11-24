import { useEffect, useState } from "react";
import type { Game } from "../../domain/game";
import { preload } from "../../images/preload";

/** Espera antes de cambiar el arte grande: al recorrer rápido no se cargan los intermedios. */
const SETTLE_MS = 140;

/**
 * Capas para el fundido cruzado: la anterior queda debajo mientras la nueva aparece encima.
 * La nueva entra cuando su imagen ya está decodificada (o se sabe que no existe).
 */
export function useLayers(game: Game | undefined, imageFor: (game: Game) => string | null): Game[] {
  const [layers, setLayers] = useState<Game[]>(game ? [game] : []);

  useEffect(() => {
    if (!game) return setLayers([]);
    let cancelled = false;
    const timer = setTimeout(() => {
      void preload(imageFor(game)).then(() => {
        if (!cancelled) setLayers((prev) => (prev.at(-1)?.id === game.id ? prev : [...prev.slice(-1), game]));
      });
    }, SETTLE_MS);
    return () => {
      cancelled = true;
      clearTimeout(timer);
    };
    // imageFor es una función de módulo y el arte solo cambia con el juego.
  }, [game?.id]);

  // Los datos del juego (color, metadatos) pueden llegar después: refrescar la capa visible.
  return game ? layers.map((l) => (l.id === game.id ? game : l)) : layers;
}
