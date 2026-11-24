import { useCallback, useEffect, useMemo, useRef, type PointerEvent } from "react";
import { launchGame, openStorePage } from "../../api/library";
import type { Game } from "../../domain/game";
import { visibleGames } from "../../features/library/format";
import { artUrl } from "../../images/artUrl";
import { preload } from "../../images/preload";
import { PAGE_JUMP, type Action } from "../../input/actions";
import { useGamepad } from "../../input/gamepad";
import { useKeyboard } from "../../input/keyboard";
import { useLibrary } from "../../state/library";
import { useUi } from "../../state/ui";
import { Backdrop } from "./Backdrop";
import { Carousel } from "./Carousel";
import { InfoPanel } from "./InfoPanel";
import { ParallaxStage } from "./ParallaxStage";
import "./hero.css";

/** Grados máximos de inclinación extra por ratón o stick derecho. */
const MAX_TILT = 5;

export function HeroView({ onError, onSearch }: { onError: (message: string) => void; onSearch: () => void }) {
  const allGames = useLibrary((s) => s.games);
  const { view, query, selectedId, select } = useUi();
  const games = useMemo(() => visibleGames(allGames, view, query), [allGames, view, query]);
  const found = games.findIndex((g) => g.id === selectedId);
  const selectedIndex = Math.max(0, found);
  const game: Game | undefined = games[selectedIndex];
  const rootRef = useRef<HTMLDivElement>(null);

  // Precarga el arte grande de los vecinos: al avanzar, el fundido no espera a la red.
  useEffect(() => {
    for (const neighbour of [games[selectedIndex - 1], games[selectedIndex + 1]]) {
      if (neighbour) void preload(artUrl(neighbour, "hero"));
    }
  }, [games, selectedIndex]);

  const moveTo = useCallback(
    (index: number) => {
      const target = games[Math.min(games.length - 1, Math.max(0, index))];
      if (target) select(target.id);
    },
    [games, select],
  );

  const onAction = (action: Action) => {
    const moves: Partial<Record<Action, number>> = {
      prev: selectedIndex - 1,
      next: selectedIndex + 1,
      prevPage: selectedIndex - PAGE_JUMP,
      nextPage: selectedIndex + PAGE_JUMP,
      first: 0,
      last: games.length - 1,
    };
    const index = moves[action];
    if (index !== undefined) return moveTo(index);
    if (action === "search") return onSearch();
    if (!game) return;
    if (action === "launch") launchGame(game.id).catch((e) => onError(`No se pudo lanzar ${game.title}: ${e}`));
    if (action === "openStore") openStorePage(game.id).catch((e) => onError(`No se pudo abrir ${game.title}: ${e}`));
  };

  const tilt = useCallback((x: number, y: number) => {
    const style = rootRef.current?.style;
    style?.setProperty("--tilt-x", `${(x * MAX_TILT).toFixed(2)}deg`);
    style?.setProperty("--tilt-y", `${(-y * MAX_TILT).toFixed(2)}deg`);
  }, []);

  useKeyboard(onAction);
  useGamepad(onAction, tilt);

  const onPointerMove = (e: PointerEvent<HTMLDivElement>) => {
    if (e.pointerType !== "mouse") return;
    tilt((e.clientX / window.innerWidth) * 2 - 1, (e.clientY / window.innerHeight) * 2 - 1);
  };

  return (
    <div className="hero" ref={rootRef} onPointerMove={onPointerMove}>
      <Backdrop game={game} />
      <div className="hero-main">
        <ParallaxStage game={game} />
        {game ? (
          <InfoPanel game={game} onError={onError} />
        ) : (
          <p className="info-empty">{query ? `Ningún juego instalado coincide con «${query}».` : "No hay juegos instalados."}</p>
        )}
      </div>
      {games.length > 0 && (
        <Carousel games={games} selectedIndex={selectedIndex} onSelect={moveTo} onActivate={() => onAction("launch")} />
      )}
    </div>
  );
}
