//! Lee una instalación de Steam y de Epic simulada en disco, con el formato real de los archivos.

use mis_juegos_core::epic::EpicProvider;
use mis_juegos_core::game::{LaunchTarget, Source};
use mis_juegos_core::library::LibraryProvider;
use mis_juegos_core::steam::SteamProvider;
use std::path::Path;

fn write(path: &Path, text: &str) {
    std::fs::create_dir_all(path.parent().unwrap()).unwrap();
    std::fs::write(path, text).unwrap();
}

fn acf(app_id: u32, name: &str, flags: u32) -> String {
    format!(
        "\"AppState\"\n{{\n\t\"appid\"\t\t\"{app_id}\"\n\t\"name\"\t\t\"{name}\"\n\t\"StateFlags\"\t\t\"{flags}\"\n\t\"installdir\"\t\t\"{name}\"\n\t\"LastPlayed\"\t\t\"1700000000\"\n}}\n"
    )
}

#[test]
fn reads_steam_libraries_manifests_and_user_playtime() {
    let tmp = tempfile::tempdir().unwrap();
    let root = tmp.path().join("Steam");
    let second = tmp.path().join("D_Juegos");
    let second_escaped = second.to_string_lossy().replace('\\', "\\\\");

    write(
        &root.join("steamapps/libraryfolders.vdf"),
        &format!(
            "\"libraryfolders\"\n{{\n\t\"contentstatsid\"\t\t\"123\"\n\t\"0\"\n\t{{\n\t\t\"path\"\t\t\"{}\"\n\t}}\n\t\"1\"\n\t{{\n\t\t\"path\"\t\t\"{second_escaped}\"\n\t}}\n}}\n",
            root.to_string_lossy().replace('\\', "\\\\")
        ),
    );
    write(&root.join("steamapps/appmanifest_570.acf"), &acf(570, "Dota 2", 4));
    write(&root.join("steamapps/appmanifest_228980.acf"), &acf(228980, "Steamworks Common Redistributables", 4));
    write(&second.join("steamapps/appmanifest_1145350.acf"), &acf(1145350, "Hades II", 6));
    write(&second.join("steamapps/appmanifest_999.acf"), "\"AppState\" { roto");
    // StateFlags 1026: descargando, todavía no jugable.
    write(&second.join("steamapps/appmanifest_400.acf"), &acf(400, "Portal", 1026));

    // SteamID64 76561197960265738 → accountid 10.
    write(
        &root.join("config/loginusers.vdf"),
        "\"users\"\n{\n\t\"76561197960265738\"\n\t{\n\t\t\"AccountName\"\t\t\"yo\"\n\t\t\"MostRecent\"\t\t\"1\"\n\t}\n}\n",
    );
    write(
        &root.join("userdata/10/config/localconfig.vdf"),
        "\"UserLocalConfigStore\" { \"Software\" { \"Valve\" { \"Steam\" { \"apps\" {
            \"570\" { \"LastPlayed\" \"1710000000\" \"Playtime\" \"5160\" }
            \"440\" { \"LastPlayed\" \"1600000000\" \"Playtime\" \"90\" }
            \"10\"  { \"LastPlayed\" \"1600000000\" }
        } } } } }",
    );

    let mut games = SteamProvider::at(root).load().unwrap();
    games.sort_by(|a, b| a.id.cmp(&b.id));
    let ids: Vec<&str> = games.iter().map(|g| g.id.as_str()).collect();
    // 440 solo está en localconfig (jugado, no instalado): no aparece.
    // 1145350 tiene StateFlags 6 (instalado con actualización pendiente): sí aparece.
    assert_eq!(ids, ["steam:1145350", "steam:570"]);

    let hades = &games[0];
    assert_eq!(hades.title, "Hades II");
    assert_eq!(hades.install_dir.as_deref(), Some(second.join("steamapps/common/Hades II").as_path()));
    assert_eq!(hades.playtime_minutes, None);

    let dota = &games[1];
    assert_eq!(dota.playtime_minutes, Some(5160));
    assert_eq!(dota.last_played, Some(1710000000));
    assert_eq!(dota.launch, LaunchTarget::Steam { app_id: 570 });
}

#[test]
fn reads_epic_manifests_skipping_dlc_and_incomplete_installs() {
    let tmp = tempfile::tempdir().unwrap();
    let item = |file: &str, name: &str, categories: &str, incomplete: bool| {
        write(
            &tmp.path().join(file),
            &format!(
                r#"{{"FormatVersion":0,"bIsIncompleteInstall":{incomplete},"DisplayName":"{name}","AppName":"{name}App",
                "CatalogNamespace":"ns","CatalogItemId":"cat","InstallLocation":"C:\\Games\\{name}",
                "LaunchExecutable":"{name}.exe","AppCategories":[{categories}]}}"#
            ),
        )
    };
    item("A.item", "Alan", r#""public","games","applications""#, false);
    item("B.item", "Pack", r#""public","addons""#, false);
    item("C.item", "Half", r#""public","games""#, true);
    write(&tmp.path().join("ignored.txt"), "x");

    let games = EpicProvider::at(tmp.path().to_path_buf()).load().unwrap();
    assert_eq!(games.len(), 1);
    let alan = &games[0];
    assert_eq!(alan.id, "epic:AlanApp");
    assert_eq!(alan.source, Source::Epic);
    assert_eq!(alan.title, "Alan");
    assert!(alan.launch.launch_uri().starts_with("com.epicgames.launcher://apps/ns%3Acat%3AAlanApp"));
}
