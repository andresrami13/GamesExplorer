import { useEffect, useState } from "react";
import { LibraryGrid } from "./features/library/LibraryGrid";
import { useLibrary } from "./state/library";

export function App() {
  const { games, status, errors, start } = useLibrary();
  const [notice, setNotice] = useState<string | null>(null);

  useEffect(() => {
    void start();
  }, [start]);

  const installed = games.filter((g) => g.installed).length;

  return (
    <main className="app">
      <header className="app-header">
        <h1>Mis juegos</h1>
        <p className="muted" aria-live="polite">
          {status === "loading" ? "Leyendo tu biblioteca…" : `${games.length} juegos · ${installed} instalados`}
        </p>
      </header>

      {[...errors, ...(notice ? [notice] : [])].map((message) => (
        <p key={message} className="notice" role="alert">
          {message}
        </p>
      ))}

      {status === "ready" && games.length === 0 && (
        <p className="muted">No se encontró Steam ni Epic Games en este equipo.</p>
      )}
      <LibraryGrid games={games} onError={setNotice} />
    </main>
  );
}
