# Mis juegos: arquitectura

Aprobada el 24/11/2026. Repositorio `GamesExplorer`; la app se llama **Mis juegos**.

Leyenda: **[V]** verificado o conocimiento consolidado · **[I]** inferencia que hay que comprobar en tu máquina.

## 1. Stack: WinUI 3 vs Tauri + React (las 5 líneas)

1. **Pasar página con curl 3D (el requisito más difícil):** WinUI Composition no deforma mallas; solo hay giro rígido de la hoja, y un curl real exige Win2D/Direct3D con shaders propios en un SwapChainPanel. En web es un plano subdividido de Three.js con un vertex shader de ~80 líneas. **Gana Tauri.** [V]
2. **Fluidez general (carrusel, disco girando, desenfoque, transiciones):** ambos llegan a 60 fps con GPU; WebView2 es Chromium compuesto por GPU y WinUI usa Composition nativa. **Empate.** [I: 60 fps reales dependen de tu GPU; se mide con el contador de FPS en el hito b]
3. **Arranque y memoria:** WinUI arranca algo antes y consume menos (WebView2 es multiproceso, típicamente 100-200 MB). Ambos caben en < 2 s con caché. **Gana WinUI, sin romper el requisito.** [I]
4. **Mando, teclado, pantalla completa, registro de Windows, lanzar steam://:** ambos lo resuelven (Gamepad API en WebView2, crate `winreg` en Rust). **Empate.** [I: Gamepad API en WebView2 la confirmo en el hito b]
5. **Empaquetado MSIX:** WinUI lo trae de serie. Tauri genera MSI/NSIS, no MSIX; habrá que envolver el .exe con `makeappx` + manifiesto propio. **Gana WinUI, coste: un script de empaquetado.** [V]

**Recomendación: Tauri 2 + React + TypeScript, Three.js (react-three-fiber) para el álbum y Framer Motion para el resto de la UI.** El requisito diferencial de la app es el curl 3D y es donde la diferencia es grande; lo que pierdes (algo de RAM y el MSIX nativo) es acotado y tiene solución. Nota: Framer Motion anima DOM, no sirve para el curl; por eso Three.js en ese modo.

## 2. Correcciones a la especificación (antes de implementar)

- **Los appmanifest_*.acf solo existen para juegos instalados.** [V] Leyendo solo eso, la biblioteca mostrará únicamente lo instalado. Para los no instalados propongo:
  - Con API key: `IPlayerService/GetOwnedGames` da la lista completa con horas. [V]
  - Sin API key: `userdata/<id>/config/localconfig.vdf` contiene apps jugadas en esta máquina con `LastPlayed` y `Playtime` [I], y las carpetas de `appcache/librarycache` revelan appids con arte en caché [I]. Los nombres de esos appids salen de la Store API. Es una aproximación, no la lista completa, y la app lo indicará.
- **Horas y "reciente" sin API key:** probablemente salen de `localconfig.vdf` [I]. Si se confirma, el orden por horas funciona sin key.
- **librarycache cambió de formato en 2024:** antes `<appid>_library_600x900.jpg` plano, ahora subcarpetas `librarycache/<appid>/...`, a veces con subcarpetas por hash. [I] Se soportarán ambos.
- **CDN:** la URL clásica `cdn.cloudflare.steamstatic.com/steam/apps/<appid>/library_600x900.jpg` devuelve 404 en algunos juegos recientes, que usan rutas con hash. [I] Respaldo: la `header_image` que devuelve `appdetails` de la Store API, y si no, un placeholder generado.
- **Límite de la Store API:** se cita habitualmente ~200 peticiones / 5 min por IP; no está documentado oficialmente. [I] Cola con 1 petición/1,5 s, backoff exponencial ante 429 y caché de 30 días.

## 2b. Epic Games y lanzar el ejecutable (v2)

**Epic se puede leer en local, solo lo instalado.** [I, formato conocido; lo confirmo con tus archivos]
- `C:\ProgramData\Epic\EpicGamesLauncher\Data\Manifests\*.item` son JSON con `DisplayName`, `InstallLocation`, `LaunchExecutable`, `CatalogNamespace`, `CatalogItemId` y `AppName`.
- Lanzar: `com.epicgames.launcher://apps/<Namespace>%3A<CatalogItemId>%3A<AppName>?action=launch&silent=true`.
- **Portadas de Epic: no hay CDN público por id como en Steam.** La API de la tienda de Epic no es pública ni está documentada, y usarla roza el scraping que pediste evitar. Propuesta: (1) si el juego también existe en Steam, se usa ese arte, emparejando por nombre; (2) SteamGridDB opcional, con API key gratuita del usuario; (3) si no hay nada, una portada generada con el icono del .exe y su color dominante.
- Los juegos de Epic no instalados no se pueden listar sin iniciar sesión en Epic, y eso queda fuera porque no guardamos credenciales.

**Lanzar el .exe directamente: no por defecto.** [V en lo general; el comportamiento exacto depende de cada juego]
- Steam: muchos juegos llaman a `SteamAPI_RestartAppIfNecessary` y se relanzan vía Steam igualmente. Otros fallan por DRM. Además, sin Steam se pierden el overlay, el conteo de horas, la sincronización de guardados en la nube y la configuración de mando de Steam Input.
- Epic: los juegos que usan Epic Online Services esperan argumentos de autenticación que pasa el launcher (`-AUTH_PASSWORD`, `-epicusername`...). Lanzados a pelo, muchos no arrancan o no conectan.
- Decisión propuesta: siempre se lanza por el URI de la plataforma (`steam://rungameid/`, `com.epicgames.launcher://`). El modo ".exe directo" solo para **juegos añadidos a mano** (GOG sin Galaxy, emuladores, juegos sin tienda). El .exe de cada juego sí se detecta, pero para sacar su icono.

**Impacto en la arquitectura:** el backend pasa de `steam::` a un trait `LibraryProvider` con tres implementaciones: `SteamProvider`, `EpicProvider` y `ManualProvider`. Cada `Game` lleva `source` y su propio `LaunchTarget`. La UI no distingue el origen salvo por un icono pequeño y el texto del botón ("Abrir en Steam" o "Abrir en Epic").

## 2c. Hero con movimiento lento "3D" (v2, según tu mockup)

Steam publica por separado `library_hero` (fondo sin logo) y `logo` (PNG transparente). Eso permite un parallax de capas real sin trucos:
1. **Capa fondo:** `library_hero` con Ken Burns muy lento (zoom de 1.00 a 1.08 y desplazamiento de ~2 %, en un ciclo de 25-40 s, ida y vuelta con easing sinusoidal).
2. **Capa logo:** se desplaza en sentido contrario y a otra velocidad, así se percibe profundidad.
3. **Marco inclinado** como en el mockup: `perspective` + `rotateY` de ±3-4° oscilando muy despacio, más una leve respuesta a la posición del ratón o del stick derecho del mando.
4. **Opcional:** encadenar 3-5 capturas de la Store API con fundido cruzado cada ~12 s, cada una con su Ken Burns.

Rendimiento: solo `transform` y `opacity`, que corren en el compositor de GPU sin relayout; 60 fps son esperables también en GPU integrada [I, se mide en el hito c]. Se respeta `prefers-reduced-motion` y habrá un interruptor "Movimiento del hero" en Ajustes. Lo que no propongo: profundidad real a partir de una sola imagen (mapa de profundidad generado con IA). Pesa mucho para el beneficio que da.

Del mockup: "41/50 logros" solo existe con API key de Steam; sin key se oculta ese dato, no se muestra vacío. El texto gris sobre morado del mockup lo validaré contra 4,5:1 al implementar.

## 3. Arquitectura

```
┌──────────────── Frontend (React + TS, WebView2) ───────────────┐
│ modes/album (R3F + shader curl) · modes/hero · modes/sleeve    │
│ library store (Zustand): búsqueda, filtros, orden, selección   │
│ input: mapa de acciones único ← teclado + Gamepad API          │
│ image pipeline: ventana visible ±1 página, precarga            │
└───────────── invoke() / eventos ─────── protocolo img:// ──────┘
┌──── src-tauri (ventana, comandos, protocolo img, enriquecedor) ─┐
│ commands   get_cached_library · refresh_library · launch_game   │
│ protocol   img://localhost/steam/<appid>/<tipo> → bytes          │
│ enrich     hilo de fondo: color + Store API → evento game-updated│
└──────────────────────────────┬──────────────────────────────────┘
┌──── crates/core (Rust puro, se prueba sin ventana) ─────────────┐
│ vdf              parser KeyValues texto                          │
│ steam::*         registro, libraryfolders, acf, localconfig      │
│ epic             manifiestos *.item       (trait LibraryProvider)│
│ manual           juegos añadidos a mano (.exe directo) · hito d  │
│ images           caché propia → librarycache → CDN, color        │
│ metadata         Store API con límite y caché; Web API · hito d  │
│ snapshot         instantánea enriquecida para arrancar rápido    │
│ persistence      JSON atómico en %APPDATA%\MisJuegos             │
└─────────────────────────────────────────────────────────────────┘
```

Decisiones clave:
- **Arranque < 2 s:** al abrir se sirve `library.json` (en %LOCALAPPDATA%\MisJuegos) (snapshot de la última lectura con colores ya calculados) y se pinta; la relectura de Steam corre en segundo plano y cada juego completado llega con el evento `game-updated`. Sin caché (primer arranque) puede tardar más, y se indica con progreso.
- **Imágenes sin pasar por IPC:** un protocolo propio `img://localhost/steam/<appid>/<tipo>` (`http://img.localhost/...` en Windows) resuelve en Rust la cadena librarycache → caché propia → CDN. El frontend solo usa URLs; nada de base64.
- **Virtualización:** solo se montan las portadas de la página actual ±1; texturas Three.js con LRU.
- **Datos de usuario:** JSON escritos de forma atómica (archivo temporal + rename). La API key se guarda en el Credential Manager de Windows (crate `keyring`), no en JSON en claro. No se piden ni guardan contraseñas.
- **Fuentes:** Fraunces e Inter empaquetadas en la app (licencia OFL), sin cargarlas de Google en tiempo de ejecución.
- **Tests:** Rust con fixtures reales de `.vdf`/`.acf` (anonimizados); Vitest para filtros, orden y mapa de entrada.

## 4. Estructura de carpetas

```
GamesExplorer/
├─ README.md                    compilar, ejecutar, empaquetar MSIX
├─ docs/ARCHITECTURE.md
├─ package.json · vite.config.ts · tsconfig.json
├─ src/                         frontend
│  ├─ main.tsx · App.tsx
│  ├─ api/                      wrappers tipados de invoke() y eventos
│  ├─ domain/                   tipos Game, Collection, Settings
│  ├─ state/                    stores Zustand (library, ui, settings)
│  ├─ features/
│  │  ├─ library/               búsqueda, filtros, orden (lógica pura + UI)
│  │  ├─ favorites/             favoritos y colecciones
│  │  └─ settings/              pantalla de ajustes
│  ├─ modes/
│  │  ├─ album/                 Book.tsx, PageCurl (shader), Sleeve slots
│  │  ├─ hero/                  HeroView, ParallaxStage, Carousel
│  │  └─ sleeve/                CdSleeve, SpinningDisc, PageTabs
│  ├─ input/                    actions.ts, keyboard.ts, gamepad.ts
│  ├─ images/                   useCover, prefetch, LRU de texturas
│  ├─ ui/                       botones, foco visible, tokens de tema
│  └─ assets/                   fuentes, sonido de página
├─ Cargo.toml                   workspace
├─ crates/core/                 lógica sin dependencias de ventana
│  ├─ src/ vdf.rs · game.rs · library.rs · snapshot.rs · persistence.rs · http.rs · paths.rs
│  │       steam/ (locator, library_folders, manifest, localconfig) · epic/
│  │       images/ (caché, color) · metadata/ (store_api, rate_limit)
│  └─ tests/library.rs          instalaciones de Steam y Epic simuladas
├─ src-tauri/                   main.rs · state.rs · commands.rs · protocol.rs · enrich.rs
└─ packaging/msix/              AppxManifest.xml, iconos, build-msix.ps1
```

## 5. Hitos

- **a)** Lectores de Steam y Epic + caché de imágenes + una cuadrícula mínima. Hecho.
- **Hero + carrusel** (adelantado a petición del usuario, antes del Álbum): solo juegos instalados, arte grande con parallax lento, carrusel con muelle a 60 fps, teclado, arrastre, rueda y mando, pestañas Instalados/Recientes y búsqueda.
- **b)** Modo Álbum con curl 3D, teclado, arrastre de esquina, sonido y mando.
- **c)** Modo Funda + disco.
- **d)** Ajustes, juegos manuales con .exe, SteamGridDB opcional, favoritos, colecciones, filtros por género, F11 y modo TV, empaquetado MSIX.

Decisiones del modo Hero:
- La pista del carrusel se mueve con un muelle propio que escribe `transform` en el DOM en cada fotograma; React solo re-renderiza cuando cambia la portada central. No se usa Framer Motion: para un solo muelle no compensa la dependencia.
- Solo se montan unas 16 portadas alrededor de la actual, sea cual sea el tamaño de la biblioteca.
- El arte grande cambia 140 ms después de la última pulsación y cuando la imagen ya está decodificada, con fundido cruzado; se precargan el anterior y el siguiente.
- Los movimientos lentos (barrido con zoom, deriva del logo, balanceo del marco) son animaciones CSS en el compositor; se desactivan con `prefers-reduced-motion`.

**Cambio respecto a la propuesta:** la lógica vive en `crates/core` y no en `src-tauri`. Así se compila y se prueba sin WebView2 (también en CI y en Linux) y `src-tauri` queda como capa fina.

## 6. Limitación de entorno

Yo trabajo en un contenedor Linux: puedo escribir el código y ejecutar los tests de Rust y del frontend, pero el ejecutable de Windows lo compilas tú (o lo compilo en tu PC con tu permiso). El README tendrá los pasos exactos.
