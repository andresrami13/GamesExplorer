import { useRef, useState, type PointerEvent, type WheelEvent } from "react";
import type { Game } from "../../domain/game";
import { artUrl } from "../../images/artUrl";
import { useCarouselMotion } from "./useCarouselMotion";

const SLOT_WIDTH = 150;
const GAP = 18;
const STEP = SLOT_WIDTH + GAP;
// Solo se montan las portadas cercanas a la posición actual (virtualización).
const RENDER_BEFORE = 3;
const RENDER_AFTER = 13;
/** Píxeles de movimiento a partir de los cuales un clic pasa a ser arrastre. */
const DRAG_THRESHOLD = 6;
/** Rueda o touchpad: delta acumulado necesario para avanzar una portada. */
const WHEEL_STEP = 60;

interface Props {
  games: Game[];
  selectedIndex: number;
  onSelect: (index: number) => void;
  onActivate: () => void;
}

export function Carousel({ games, selectedIndex, onSelect, onActivate }: Props) {
  const [center, setCenter] = useState(selectedIndex);
  const motion = useCarouselMotion({ target: selectedIndex, count: games.length, step: STEP, onCenter: setCenter });
  const drag = useRef<{ id: number; x: number; from: number; samples: { x: number; t: number }[]; active: boolean } | null>(null);
  const wheel = useRef(0);

  const onPointerDown = (e: PointerEvent<HTMLDivElement>) => {
    if (e.button !== 0) return;
    drag.current = { id: e.pointerId, x: e.clientX, from: 0, samples: [{ x: e.clientX, t: e.timeStamp }], active: false };
  };

  const onPointerMove = (e: PointerEvent<HTMLDivElement>) => {
    const d = drag.current;
    if (!d || d.id !== e.pointerId) return;
    const dx = e.clientX - d.x;
    if (!d.active) {
      if (Math.abs(dx) < DRAG_THRESHOLD) return;
      // Captura solo al empezar a arrastrar: un clic normal sigue llegando a la portada.
      d.active = true;
      d.from = motion.grab();
      e.currentTarget.setPointerCapture(e.pointerId);
    }
    d.samples = [...d.samples.filter((s) => e.timeStamp - s.t < 100), { x: e.clientX, t: e.timeStamp }];
    motion.dragTo(d.from - dx / STEP);
  };

  const onPointerUp = (e: PointerEvent<HTMLDivElement>) => {
    const d = drag.current;
    drag.current = null;
    if (!d?.active || d.id !== e.pointerId) return;
    const first = d.samples[0];
    const elapsed = (e.timeStamp - first.t) / 1000;
    const velocity = elapsed > 0 ? -(e.clientX - first.x) / STEP / elapsed : 0;
    onSelect(motion.release(velocity));
  };

  const onWheel = (e: WheelEvent<HTMLDivElement>) => {
    wheel.current += Math.abs(e.deltaX) > Math.abs(e.deltaY) ? e.deltaX : e.deltaY;
    const steps = Math.trunc(wheel.current / WHEEL_STEP);
    if (steps === 0) return;
    wheel.current -= steps * WHEEL_STEP;
    onSelect(Math.min(games.length - 1, Math.max(0, selectedIndex + steps)));
  };

  const from = Math.max(0, center - RENDER_BEFORE);
  const to = Math.min(games.length, center + RENDER_AFTER);
  const selected = games[selectedIndex];

  return (
    <div className="carousel">
      <div
        className="carousel-viewport"
        role="listbox"
        tabIndex={0}
        aria-label="Juegos"
        aria-activedescendant={selected ? optionId(selected) : undefined}
        onPointerDown={onPointerDown}
        onPointerMove={onPointerMove}
        onPointerUp={onPointerUp}
        onPointerCancel={onPointerUp}
        onWheel={onWheel}
      >
        <div className="carousel-track" ref={motion.trackRef}>
          {games.slice(from, to).map((game, offset) => {
            const index = from + offset;
            const isSelected = index === selectedIndex;
            return (
              <div
                key={game.id}
                id={optionId(game)}
                role="option"
                aria-selected={isSelected}
                className={isSelected ? "slot slot-selected" : "slot"}
                style={{ transform: `translate3d(${index * STEP}px,0,0)`, width: SLOT_WIDTH }}
                onClick={() => (isSelected ? onActivate() : onSelect(index))}
              >
                <Cover game={game} />
              </div>
            );
          })}
        </div>
      </div>
      <div className="carousel-progress" aria-hidden="true">
        <div className="carousel-progress-fill" ref={motion.progressRef} />
      </div>
    </div>
  );
}

const optionId = (game: Game) => `game-${game.id}`;

function Cover({ game }: { game: Game }) {
  const url = artUrl(game, "capsule");
  const [failed, setFailed] = useState(false);
  return url && !failed ? (
    <img src={url} alt={game.title} loading="lazy" decoding="async" draggable={false} onError={() => setFailed(true)} />
  ) : (
    <span className="slot-placeholder" style={{ background: `linear-gradient(160deg, ${game.color ?? "#4a2368"}, #120c1a 85%)` }}>
      {game.title}
    </span>
  );
}
