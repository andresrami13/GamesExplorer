import { useEffect, useRef } from "react";
import type { Action } from "./actions";

const KEYS: Record<string, Action> = {
  ArrowLeft: "prev",
  ArrowRight: "next",
  PageUp: "prevPage",
  PageDown: "nextPage",
  Home: "first",
  End: "last",
  Enter: "launch",
  "/": "search",
};

/**
 * Teclado global. Dentro de un campo de texto solo actúan Escape (sale del campo)
 * y las flechas izquierda/derecha cuando el campo está vacío.
 */
export function useKeyboard(onAction: (action: Action) => void) {
  const handler = useRef(onAction);
  handler.current = onAction;

  useEffect(() => {
    const onKeyDown = (e: KeyboardEvent) => {
      if (e.altKey || e.ctrlKey || e.metaKey) return;
      const field = e.target instanceof HTMLInputElement ? e.target : null;
      if (field) {
        if (e.key === "Escape") field.blur();
        if (field.value || !(e.key === "ArrowLeft" || e.key === "ArrowRight")) return;
      }
      // Enter sobre un botón enfocado lo pulsa el propio botón.
      if (e.key === "Enter" && e.target instanceof HTMLButtonElement) return;
      const action = KEYS[e.key];
      if (!action) return;
      e.preventDefault();
      handler.current(action);
    };
    window.addEventListener("keydown", onKeyDown);
    return () => window.removeEventListener("keydown", onKeyDown);
  }, []);
}
