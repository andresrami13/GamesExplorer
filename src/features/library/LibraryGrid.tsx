import type { Game } from "../../domain/game";
import { GameCard } from "./GameCard";

export function LibraryGrid({ games, onError }: { games: Game[]; onError: (message: string) => void }) {
  return (
    <ul className="grid" aria-label="Biblioteca">
      {games.map((game) => (
        <li key={game.id}>
          <GameCard game={game} onError={onError} />
        </li>
      ))}
    </ul>
  );
}
