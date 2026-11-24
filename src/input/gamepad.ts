import { useEffect, useRef } from "react";
import type { Action } from "./actions";

// Botones del mapeo estándar de la Gamepad API (Xbox: A, X, LB, RB, cruceta).
const BUTTONS: Record<number, Action> = { 0: "launch", 2: "openStore", 4: "prevPage", 5: "nextPage", 14: "prev", 15: "next" };
const REPEATING = new Set<Action>(["prev", "next", "prevPage", "nextPage"]);
const STICK_DEADZONE = 0.5;
const LOOK_DEADZONE = 0.15;
const REPEAT_DELAY_MS = 380;
const REPEAT_EVERY_MS = 110;

/**
 * Mando: cruceta o stick izquierdo para moverse (mantener pulsado repite),
 * A para jugar, X para abrir la tienda, LB/RB para saltar. El stick derecho inclina el hero.
 * Solo sondea mientras hay un mando conectado.
 */
export function useGamepad(onAction: (action: Action) => void, onLook: (x: number, y: number) => void) {
  const handlers = useRef({ onAction, onLook });
  handlers.current = { onAction, onLook };

  useEffect(() => {
    let raf = 0;
    const held = new Map<Action, number>(); // acción → próximo instante de repetición

    const poll = (now: number) => {
      const pad = navigator.getGamepads().find((p) => p?.connected);
      if (!pad) {
        raf = 0;
        return;
      }
      const pressed = new Set<Action>();
      pad.buttons.forEach((b, i) => {
        const action = BUTTONS[i];
        if (action && b.pressed) pressed.add(action);
      });
      const x = pad.axes[0] ?? 0;
      if (x < -STICK_DEADZONE) pressed.add("prev");
      if (x > STICK_DEADZONE) pressed.add("next");

      for (const action of pressed) {
        const next = held.get(action);
        if (next === undefined) {
          handlers.current.onAction(action);
          held.set(action, REPEATING.has(action) ? now + REPEAT_DELAY_MS : Infinity);
        } else if (now >= next) {
          handlers.current.onAction(action);
          held.set(action, now + REPEAT_EVERY_MS);
        }
      }
      for (const action of held.keys()) if (!pressed.has(action)) held.delete(action);

      const dead = (v: number) => (Math.abs(v) < LOOK_DEADZONE ? 0 : v);
      handlers.current.onLook(dead(pad.axes[2] ?? 0), dead(pad.axes[3] ?? 0));
      raf = requestAnimationFrame(poll);
    };

    const start = () => {
      if (!raf) raf = requestAnimationFrame(poll);
    };
    window.addEventListener("gamepadconnected", start);
    start(); // por si el mando ya estaba conectado al abrir
    return () => {
      window.removeEventListener("gamepadconnected", start);
      cancelAnimationFrame(raf);
    };
  }, []);
}
