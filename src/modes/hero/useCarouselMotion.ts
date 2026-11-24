import { useCallback, useEffect, useRef } from "react";

// Muelle casi crítico: llega rápido y sin rebote visible.
const STIFFNESS = 210;
const DAMPING = 2 * Math.sqrt(STIFFNESS) * 0.92;
const MAX_DT = 1 / 30;
const REST = 0.0005;
/** Al soltar un arrastre, cuánto "lanza" la velocidad (segundos de inercia proyectada). */
const FLING_SECONDS = 0.18;
/** Resistencia al arrastrar más allá de los extremos. */
const EDGE_RESISTANCE = 0.3;

const reducedMotion = () => window.matchMedia("(prefers-reduced-motion: reduce)").matches;

interface Options {
  target: number;
  count: number;
  /** Píxeles entre una portada y la siguiente. */
  step: number;
  /** Se llama cuando la posición animada cruza a otra portada, para mover la ventana renderizada. */
  onCenter: (index: number) => void;
}

/**
 * Mueve la pista del carrusel con un muelle a 60 fps escribiendo `transform` directamente
 * en el DOM: React no re-renderiza en cada fotograma, solo cuando cambia la portada central.
 */
export function useCarouselMotion({ target, count, step, onCenter }: Options) {
  const trackRef = useRef<HTMLDivElement>(null);
  const progressRef = useRef<HTMLDivElement>(null);
  const m = useRef({ pos: target, vel: 0, target, dragging: false, raf: 0, last: 0, center: target });
  const config = useRef({ count, step, onCenter });
  config.current = { count, step, onCenter };

  const write = useCallback(() => {
    const { pos } = m.current;
    const { count, step, onCenter } = config.current;
    if (trackRef.current) trackRef.current.style.transform = `translate3d(${-pos * step}px,0,0)`;
    if (progressRef.current) {
      const fraction = count > 1 ? Math.min(1, Math.max(0, pos / (count - 1))) : 1;
      progressRef.current.style.transform = `scaleX(${0.08 + 0.92 * fraction})`;
    }
    const center = Math.round(pos);
    if (center !== m.current.center) {
      m.current.center = center;
      onCenter(center);
    }
  }, []);

  const tick = useCallback(
    (now: number) => {
      const s = m.current;
      const dt = Math.min(MAX_DT, (now - s.last) / 1000);
      s.last = now;
      if (!s.dragging) {
        const accel = -STIFFNESS * (s.pos - s.target) - DAMPING * s.vel;
        s.vel += accel * dt;
        s.pos += s.vel * dt;
        if (Math.abs(s.pos - s.target) < REST && Math.abs(s.vel) < REST) {
          s.pos = s.target;
          s.vel = 0;
        }
      }
      write();
      s.raf = s.dragging || s.pos !== s.target ? requestAnimationFrame(tick) : 0;
    },
    [write],
  );

  const kick = useCallback(() => {
    const s = m.current;
    if (s.raf) return;
    s.last = performance.now();
    s.raf = requestAnimationFrame(tick);
  }, [tick]);

  useEffect(() => {
    const s = m.current;
    s.target = target;
    if (reducedMotion()) {
      s.pos = target;
      s.vel = 0;
      write();
    } else {
      kick();
    }
  }, [target, kick, write]);

  useEffect(() => write(), [step, count, write]);
  useEffect(() => {
    const s = m.current;
    return () => {
      cancelAnimationFrame(s.raf);
      s.raf = 0; // StrictMode desmonta y vuelve a montar: sin esto `kick` creería que sigue en marcha.
    };
  }, []);

  /** Empieza un arrastre: devuelve la posición de partida. */
  const grab = useCallback(() => {
    m.current.dragging = true;
    m.current.vel = 0;
    kick();
    return m.current.pos;
  }, [kick]);

  const dragTo = useCallback((pos: number) => {
    const max = config.current.count - 1;
    const over = pos < 0 ? pos : pos > max ? pos - max : 0;
    m.current.pos = pos - over * (1 - EDGE_RESISTANCE);
  }, []);

  /** Suelta con la velocidad del gesto (portadas/s) y devuelve la portada donde se detendrá. */
  const release = useCallback((velocity: number) => {
    const s = m.current;
    s.dragging = false;
    s.vel = velocity;
    const max = config.current.count - 1;
    return Math.min(max, Math.max(0, Math.round(s.pos + velocity * FLING_SECONDS)));
  }, []);

  return { trackRef, progressRef, grab, dragTo, release };
}
