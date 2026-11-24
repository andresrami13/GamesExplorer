import { useEffect, useRef, useState } from "react";
import { TopBar } from "./features/library/TopBar";
import { HeroView } from "./modes/hero/HeroView";
import { useLibrary } from "./state/library";

export function App() {
  const { games, status, errors, start } = useLibrary();
  const [notice, setNotice] = useState<string | null>(null);
  const searchRef = useRef<HTMLInputElement>(null);

  useEffect(() => {
    void start();
  }, [start]);

  useEffect(() => {
    if (!notice) return;
    const timer = setTimeout(() => setNotice(null), 6000);
    return () => clearTimeout(timer);
  }, [notice]);

  const messages = [...errors, ...(notice ? [notice] : [])];

  return (
    <main className="app">
      <TopBar count={games.length} searchRef={searchRef} />
      {status === "loading" ? (
        <p className="loading" aria-live="polite">
          Leyendo tu biblioteca…
        </p>
      ) : (
        <HeroView onError={setNotice} onSearch={() => searchRef.current?.focus()} />
      )}
      {messages.length > 0 && (
        <div className="notices">
          {messages.map((message) => (
            <p key={message} className="notice" role="alert">
              {message}
            </p>
          ))}
        </div>
      )}
    </main>
  );
}
