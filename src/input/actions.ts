/** Acciones de navegación, independientes del dispositivo que las produce. */
export type Action = "prev" | "next" | "prevPage" | "nextPage" | "first" | "last" | "launch" | "openStore" | "search";

/** Cuántos juegos salta "página anterior/siguiente" (RePág/AvPág, LB/RB). */
export const PAGE_JUMP = 5;
