import { useState, type CSSProperties } from "react";
import { launchGame } from "../../api/library";
import type { Game } from "../../domain/game";
import { artUrl } from "../../images/artUrl";
import { displayTitle, formatPlaytime } from "./format";

const SOURCE_LABEL = { steam: "Steam", epic: "Epic" } as const;

export function GameCard({ game, onError }: { game: Game; onError: (message: string) => void }) {
  const title = displayTitle(game);
  const cover = artUrl(game, "capsule");
  const [coverFailed, setCoverFailed] = useState(false);
  const meta = [SOURCE_LABEL[game.source], game.installed ? "Instalado" : "No instalado", formatPlaytime(game.playtimeMinutes)]
    .filter(Boolean)
    .join(" · ");

  return (
    <button
      type="button"
      className="card"
      aria-label={`Jugar a ${title}. ${meta}`}
      style={{ "--accent": game.color ?? "var(--surface-2)" } as CSSProperties}
      onClick={() => launchGame(game.id).catch((e) => onError(`No se pudo lanzar ${title}: ${e}`))}
    >
      <span className="card-cover">
        {cover && !coverFailed ? (
          <img src={cover} alt="" loading="lazy" decoding="async" onError={() => setCoverFailed(true)} />
        ) : (
          <span className="card-placeholder">{title}</span>
        )}
      </span>
      <span className="card-title">{title}</span>
      <span className="card-meta">{meta}</span>
    </button>
  );
}
