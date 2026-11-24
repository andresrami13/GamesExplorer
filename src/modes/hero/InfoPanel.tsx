import { launchGame, openStorePage } from "../../api/library";
import type { Game } from "../../domain/game";
import { eyebrow, formatLastPlayed, formatPlaytime } from "../../features/library/format";

const STORE_LABEL = { steam: "Abrir en Steam", epic: "Abrir en Epic" } as const;

export function InfoPanel({ game, onError }: { game: Game; onError: (message: string) => void }) {
  const stats = [
    { value: formatPlaytime(game.playtimeMinutes), label: "jugadas" },
    { value: formatLastPlayed(game.lastPlayed), label: "último uso" },
  ].filter((s) => s.value);
  const kicker = eyebrow(game);
  const fail = (what: string) => (e: unknown) => onError(`No se pudo ${what} ${game.title}: ${e}`);

  return (
    // La clave reinicia la animación de entrada en cada cambio de juego.
    <section key={game.id} className="info" aria-live="polite" aria-label={game.title}>
      {kicker && <p className="info-kicker">{kicker}</p>}
      <h2 className="info-title">{game.title}</h2>
      {game.metadata?.description && <p className="info-description">{game.metadata.description}</p>}
      {stats.length > 0 && (
        <dl className="info-stats">
          {stats.map((s) => (
            <div key={s.label}>
              <dt>{s.label}</dt>
              <dd>{s.value}</dd>
            </div>
          ))}
        </dl>
      )}
      <div className="info-actions">
        <button type="button" className="btn btn-primary" onClick={() => launchGame(game.id).catch(fail("lanzar"))}>
          Jugar
        </button>
        <button type="button" className="btn btn-secondary" onClick={() => openStorePage(game.id).catch(fail("abrir"))}>
          {STORE_LABEL[game.source]}
        </button>
      </div>
    </section>
  );
}
