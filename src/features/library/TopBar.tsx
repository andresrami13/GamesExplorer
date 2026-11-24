import type { RefObject } from "react";
import { useUi } from "../../state/ui";
import type { View } from "./format";

const VIEWS: { id: View; label: string }[] = [
  { id: "installed", label: "Instalados" },
  { id: "recent", label: "Recientes" },
];

export function TopBar({ count, searchRef }: { count: number; searchRef: RefObject<HTMLInputElement | null> }) {
  const { view, query, setView, setQuery } = useUi();
  return (
    <header className="topbar">
      <h1 className="brand">Mis juegos</h1>
      <nav className="topbar-views" aria-label="Vistas">
        {VIEWS.map((v) => (
          <button key={v.id} type="button" className="pill" aria-pressed={view === v.id} onClick={() => setView(v.id)}>
            {v.label}
          </button>
        ))}
      </nav>
      <input
        ref={searchRef}
        className="search"
        type="search"
        aria-label="Buscar juegos"
        placeholder={`Buscar en ${count} juegos…`}
        value={query}
        onChange={(e) => setQuery(e.target.value)}
      />
    </header>
  );
}
