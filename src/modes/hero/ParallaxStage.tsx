import { useState, type CSSProperties } from "react";
import type { Game } from "../../domain/game";
import { artUrl } from "../../images/artUrl";
import { useLayers } from "./useLayers";

const hero = (game: Game) => artUrl(game, "hero");

/**
 * Marco inclinado con el arte grande. Tres movimientos lentos superpuestos dan la
 * sensación de profundidad: el fondo hace un barrido con zoom, el logo deriva en
 * sentido contrario y el marco entero se balancea. Todo con `transform` (GPU).
 * La inclinación por ratón o stick llega como variables CSS en `.stage-tilt`.
 */
export function ParallaxStage({ game }: { game: Game | undefined }) {
  const layers = useLayers(game, hero);
  return (
    <div className="stage" aria-hidden="true">
      <div className="stage-tilt">
        <div className="stage-frame">
          {layers.map((g) => (
            <StageLayer key={g.id} game={g} />
          ))}
        </div>
      </div>
    </div>
  );
}

function StageLayer({ game }: { game: Game }) {
  // Sin arte "hero" se usa la cabecera; sin ninguna, queda el degradado del color.
  const sources = [artUrl(game, "hero"), artUrl(game, "header")].filter((u): u is string => u !== null);
  const [attempt, setAttempt] = useState(0);
  const [logoFailed, setLogoFailed] = useState(false);
  const logo = artUrl(game, "logo");
  const src = sources[attempt];

  return (
    <div className="stage-layer" style={{ "--accent": game.color ?? "#4a2368" } as CSSProperties}>
      {src && <img className="stage-art" src={src} alt="" onError={() => setAttempt((a) => a + 1)} />}
      {logo && !logoFailed && <img className="stage-logo" src={logo} alt="" onError={() => setLogoFailed(true)} />}
    </div>
  );
}
