# Mis juegos

Tu biblioteca de **Steam** y **Epic Games** como un álbum. Lee todo en local, sin pedir
usuario ni contraseña, y lanza los juegos a través de su launcher.

Estado: **hito (a)**. Ya funcionan la lectura de la biblioteca, la caché de imágenes, el color
dominante y los metadatos de la tienda, con una cuadrícula mínima para verlos. Los modos
Álbum, Hero y Funda llegan en los hitos siguientes ([arquitectura](docs/ARCHITECTURE.md)).

## Requisitos (Windows 10/11)

1. **Rust** estable con toolchain MSVC: <https://rustup.rs>
2. **Visual Studio Build Tools 2022** con la carga de trabajo *Desarrollo para el escritorio con C++*.
3. **Node.js 22** o superior.
4. **WebView2**: viene con Windows 11; en Windows 10 el instalador lo descarga si falta.

## Ejecutar en desarrollo

```powershell
npm install
npm run tauri dev
```

La primera compilación de Rust tarda varios minutos; las siguientes, segundos.

## Compilar instaladores

```powershell
npm run tauri build
```

Genera `target\release\bundle\nsis\*.exe` y `target\release\bundle\msi\*.msi`.
El paquete MSIX se añade en el hito (d).

## Tests

```powershell
cargo test --workspace   # parser VDF, Steam, Epic, caché, color, Store API
npm test                 # formato y URLs de imágenes del frontend
```

## De dónde salen los datos

| Dato | Origen |
| --- | --- |
| Carpeta de Steam | Registro: `HKCU\Software\Valve\Steam\SteamPath` |
| Juegos instalados de Steam | `steamapps\libraryfolders.vdf` y `appmanifest_*.acf` de cada biblioteca |
| Horas y última sesión | `userdata\<id>\config\localconfig.vdf` del último usuario que inició sesión |
| Juegos instalados de Epic | `C:\ProgramData\Epic\EpicGamesLauncher\Data\Manifests\*.item` |
| Portadas de Steam | Caché propia → `appcache\librarycache` de Steam → CDN público de Steam |
| Nombre, descripción, género y año | Steam Store API pública (`appdetails`), 1 petición cada 1,5 s como máximo |

Archivos propios de la app:

- `%LOCALAPPDATA%\MisJuegos\` — caché (imágenes, metadatos, instantánea de la biblioteca). Se puede borrar sin perder nada.
- `%APPDATA%\MisJuegos\` — datos del usuario (favoritos y colecciones, a partir del hito d).

Variables de entorno para pruebas: `MIS_JUEGOS_STEAM_ROOT` fuerza la carpeta de Steam y
`MIS_JUEGOS_EPIC_MANIFESTS` la carpeta de manifiestos de Epic.

## Limitaciones conocidas

- Sin API key solo se ven los juegos **instalados** y los **jugados en este PC**: Steam no guarda en local la lista completa de juegos comprados. La API key opcional llega en el hito (d).
- Los juegos de Steam jugados pero no instalados aparecen como "App 1234" hasta que la Store API devuelve su nombre.
- Epic no tiene un CDN público de portadas. De momento sus juegos muestran una portada con el título; las alternativas llegan en el hito (d).
