import type { CSSProperties } from "react";
import type { Game } from "../../domain/game";
import { artUrl } from "../../images/artUrl";
import { useLayers } from "./useLayers";

const capsule = (game: Game) => artUrl(game, "capsule");

/** Fondo de pantalla completa: color dominante + la portada muy desenfocada. */
export function Backdrop({ game }: { game: Game | undefined }) {
  const layers = useLayers(game, capsule);
  return (
    <div className="backdrop" aria-hidden="true">
      {layers.map((g) => {
        const url = capsule(g);
        return (
          <div key={g.id} className="backdrop-layer" style={{ "--accent": g.color ?? "#4a2368" } as CSSProperties}>
            {url && <img src={url} alt="" />}
          </div>
        );
      })}
      <div className="backdrop-shade" />
    </div>
  );
}
