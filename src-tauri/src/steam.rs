use std::fs;
use std::path::{Path, PathBuf};
use serde::Deserialize;
use winreg::enums::*;
use winreg::RegKey;
use keyvalues_serde;

fn get_steam_default_path() -> Option<PathBuf> {
    if let Ok(hkey) = RegKey::predef(HKEY_CURRENT_USER)
        .open_subkey("Software\\Valve\\Steam")
    {
        if let Ok(path_str) = hkey.get_value::<String, _>("SteamPath") {
            let path = PathBuf::from(path_str);
            if path.exists() {
                return Some(path);
            }
        }
    }
    let default = r"C:\Program Files (x86)\Steam";
    let path = PathBuf::from(default);
    if path.exists() {
        Some(path)
    } else {
        None
    }
}

#[derive(Deserialize)]
#[serde(untagged)]
enum LibraryFolderEntry {
    Entry { path: Option<String> },
    Other(()),
}

fn get_steam_library_paths(steam_path: &Path) -> Vec<PathBuf> {
    let mut libs = vec![steam_path.to_path_buf()];
    let vdf_path = steam_path.join("config").join("libraryfolders.vdf");
    if let Ok(content) = std::fs::read_to_string(&vdf_path) {
        if let Ok(map) = keyvalues_serde::from_str::<std::collections::HashMap<String, LibraryFolderEntry>>(&content) {
            for (_k, v) in map.iter() {
                if let LibraryFolderEntry::Entry { path: Some(path) } = v {
                    let lib_path = PathBuf::from(path);
                    if lib_path.exists() && !libs.contains(&lib_path) {
                        libs.push(lib_path);
                    }
                }
            }
        }
    }
    libs
}

#[derive(Deserialize)]
struct AppManifest {
    #[serde(rename = "AppState")]
    app_state: Option<AppState>,
}

#[derive(Deserialize)]
struct AppState {
    #[serde(rename = "appid")]
    appid: Option<StringOrNumber>,
    #[serde(rename = "installdir")]
    installdir: Option<StringOrNumber>,
}

#[derive(Deserialize, Debug)]
#[serde(untagged)]
enum StringOrNumber {
    String(String),
    Number(i64),
}

impl StringOrNumber {
    fn as_str(&self) -> Option<String> {
        match self {
            StringOrNumber::String(s) => Some(s.clone()),
            StringOrNumber::Number(n) => Some(n.to_string()),
        }
    }
}

fn get_game_paths_from_library(library_path: &Path) -> Vec<(String, PathBuf)> {
    let mut games = Vec::new();
    let steamapps = library_path.join("steamapps");
    if let Ok(entries) = fs::read_dir(&steamapps) {
        for entry in entries.flatten() {
            let path = entry.path();
            if path.is_file() && path.file_name().unwrap_or_default().to_string_lossy().starts_with("appmanifest_") {
                if let Ok(content) = fs::read_to_string(&path) {
                    if let Ok(app_state) = keyvalues_serde::from_str::<AppState>(&content) {
                        if let (Some(appid), Some(installdir)) = (app_state.appid, app_state.installdir) {
                            if let (Some(appid_str), Some(installdir_str)) = (appid.as_str(), installdir.as_str()) {
                                let game_path = steamapps.join("common").join(&installdir_str);
                                if game_path.exists() {
                                    games.push((appid_str, game_path));
                                }
                            }
                        }
                    }
                }
            }
        }
    }
    games
}

#[tauri::command]
pub fn list_steam_game_paths() -> Vec<[String; 2]> {
    let mut result = Vec::new();
    if let Some(steam_path) = get_steam_default_path() {
        let libs = get_steam_library_paths(&steam_path);
        for lib in libs {
            let games = get_game_paths_from_library(&lib);
            for (appid, path) in games {
                result.push([appid, path.to_string_lossy().to_string()]);
            }
        }
    }
    result
}