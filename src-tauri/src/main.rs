// Sin consola en release.
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

mod commands;
mod enrich;
mod protocol;
mod state;

fn main() {
    tauri::Builder::default()
        .plugin(tauri_plugin_opener::init())
        .manage(state::AppState::new())
        .register_asynchronous_uri_scheme_protocol(protocol::SCHEME, protocol::handle)
        .invoke_handler(tauri::generate_handler![
            commands::get_cached_library,
            commands::refresh_library,
            commands::launch_game,
            commands::open_store_page,
        ])
        .run(tauri::generate_context!())
        .expect("no se pudo iniciar Mis juegos");
}
